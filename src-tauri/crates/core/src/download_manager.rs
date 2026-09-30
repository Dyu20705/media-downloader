use chrono::Local;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::diagnostics::DiagnosticsBuffer;
use crate::execution::compile_acquisition_args;
use crate::media_verifier::{resolve_final_download_path, verify_and_inspect_media};
use crate::path_validator::validate_and_ensure_directory;
use crate::planner::AcquisitionPlanner;
use crate::process_runner::ProcessHandle;
use crate::progress_parser::{parse_progress_line, ParsedLineEvent};
use crate::settings::SettingsManager;
use crate::state_machine::DownloadStateMachine;
use crate::tools::ToolResolver;
use crate::types::{
    AcquisitionOperation, DownloadJob, DownloadProgress, DownloadStatus, StartDownloadRequest,
};
use crate::url_validator::validate_media_url_network;

pub struct ActiveJobHandle {
    pub job_id: String,
    pub cancel_sender: mpsc::Sender<()>,
}

pub struct DownloadManager {
    tool_resolver: Arc<ToolResolver>,
    diagnostics: Arc<DiagnosticsBuffer>,
    settings: Arc<SettingsManager>,
    active_job: Arc<RwLock<Option<DownloadJob>>>,
    active_handle: Arc<Mutex<Option<ActiveJobHandle>>>,
    admission_lock: Arc<Mutex<()>>,
}

impl DownloadManager {
    pub fn new(
        tool_resolver: Arc<ToolResolver>,
        diagnostics: Arc<DiagnosticsBuffer>,
        settings: Arc<SettingsManager>,
    ) -> Self {
        Self {
            tool_resolver,
            diagnostics,
            settings,
            active_job: Arc::new(RwLock::new(None)),
            active_handle: Arc::new(Mutex::new(None)),
            admission_lock: Arc::new(Mutex::new(())),
        }
    }

    pub async fn get_active_job(&self) -> Option<DownloadJob> {
        self.active_job.read().await.clone()
    }

    pub async fn start_download(
        &self,
        request: StartDownloadRequest,
    ) -> Result<DownloadJob, String> {
        // Admission, process spawn, and slot publication are one atomic operation.
        let _admission_guard = self.admission_lock.lock().await;

        if let Some(existing) = self.get_active_job().await {
            if matches!(
                existing.status,
                DownloadStatus::Downloading
                    | DownloadStatus::PostProcessing
                    | DownloadStatus::Verifying
                    | DownloadStatus::Cancelling
            ) {
                return Err(
                    "A download is already in progress. Only 1 concurrent download is permitted."
                        .to_string(),
                );
            }
        }

        // Bind admission to the plan the user reviewed, before filesystem or network work.
        let source_graph = crate::media_graph::MediaGraph::build_source_graph(
            &request.metadata,
            request.metadata.source_type.unwrap_or_default(),
        );
        let current_settings = self.settings.get_settings();
        let plan = AcquisitionPlanner::plan_with_policy(
            &source_graph,
            &request.acquisition,
            (&current_settings).into(),
        )?;
        if plan.id != request.expected_plan_id {
            return Err(
                "Acquisition plan changed. Review the updated plan before downloading.".into(),
            );
        }
        for tool in required_tools(&plan)? {
            self.tool_resolver.resolve_tool_with_settings(tool, Some(&current_settings))
                .await.ok_or_else(|| format!("Required tool {tool} is unavailable. Install or repair it before downloading."))?;
        }

        // 2. Validate URL and Output Directory
        let valid_url = validate_media_url_network(&request.metadata.webpage_url)
            .await
            .map_err(|e| e.to_string())?;
        let mut output_dir_path =
            validate_and_ensure_directory(&request.acquisition.output_directory)
                .map_err(|e| e.to_string())?;

        // 3. Resolve yt-dlp tool
        let ytdlp_tool = self
            .tool_resolver
            .resolve_tool_with_settings("yt-dlp", Some(&current_settings))
            .await
            .ok_or_else(|| "yt-dlp executable not found".to_string())?;

        let mut acquisition = request.acquisition.clone();
        acquisition.output_directory = output_dir_path.to_string_lossy().to_string();
        let sidecar = matches!(
            plan.operation,
            AcquisitionOperation::ThumbnailOnly | AcquisitionOperation::SubtitlesOnly
        );
        {
            let unique = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_nanos();
            output_dir_path = output_dir_path.join(format!("{}-{unique}", plan.id));
            std::fs::create_dir(&output_dir_path).map_err(|e| e.to_string())?;
            acquisition.output_directory = output_dir_path.to_string_lossy().into_owned();
        }
        let compiled = compile_acquisition_args(
            &plan,
            &crate::execution::ExecutionContext::new(
                &acquisition.output_directory,
                &current_settings,
            ),
        )?;

        let job_id = format!("job-{}", Local::now().timestamp_millis());
        let now_str = Local::now().to_rfc3339();

        let initial_job = DownloadJob {
            id: job_id.clone(),
            url: valid_url.clone(),
            output_directory: output_dir_path.to_string_lossy().to_string(),
            status: DownloadStatus::Downloading,
            progress: DownloadProgress::default(),
            metadata: request.metadata.clone(),
            final_file_name: None,
            final_file_path: None,
            inspection: None,
            verification: None,
            acquisition_plan: plan.clone(),
            error_message: None,
            created_at: now_str,
            completed_at: None,
            intent: None,
            recipe: None,
            fingerprint: None,
            explainable_result: None,
        };

        self.diagnostics.log(
            "INFO",
            "DOWNLOAD_MANAGER",
            &format!(
                "Starting download for '{}' using acquisition plan {}",
                request.metadata.title, plan.id
            ),
        );

        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

        let mut final_args = compiled.arguments.clone();
        if let Some(ffmpeg_tool) = self
            .tool_resolver
            .resolve_tool_with_settings("ffmpeg", Some(&current_settings))
            .await
        {
            final_args.push("--ffmpeg-location".to_string());
            final_args.push(ffmpeg_tool.path.to_string_lossy().to_string());
        }

        // Spawn process runner
        let mut proc_handle =
            ProcessHandle::spawn_with_streaming(&ytdlp_tool.path, &final_args, line_tx)
                .await
                .inspect_err(|e| {
                    self.diagnostics.log("ERROR", "PROCESS", e);
                })?;

        // Publish the reserved slot only after the process was spawned successfully.
        *self.active_job.write().await = Some(initial_job.clone());

        // Save active handle
        *self.active_handle.lock().await = Some(ActiveJobHandle {
            job_id: job_id.clone(),
            cancel_sender: cancel_tx,
        });

        // Spawn background task to monitor process and parse progress
        let active_job_clone = self.active_job.clone();
        let diagnostics_clone = self.diagnostics.clone();
        let tool_resolver_clone = self.tool_resolver.clone();
        let active_handle_clone = self.active_handle.clone();
        let media_id = request.metadata.id.clone();
        let is_lossy_warning = compiled.is_lossy_conversion;
        let verification_settings = current_settings.clone();

        tokio::spawn(async move {
            let mut state_machine = DownloadStateMachine::with_state(DownloadStatus::Downloading);
            let mut last_progress_emit = Instant::now() - Duration::from_millis(500);
            let mut captured_destination: Option<String> = None;
            let mut is_cancelled = false;

            loop {
                tokio::select! {
                    _ = cancel_rx.recv() => {
                        is_cancelled = true;
                        diagnostics_clone.log("WARN", "DOWNLOAD_MANAGER", "Cancellation received, terminating process tree...");
                        let _ = state_machine.transition(DownloadStatus::Cancelling);

                        if let Some(ref mut job) = *active_job_clone.write().await {
                            job.status = DownloadStatus::Cancelling;
                        }

                        proc_handle.kill_tree().await;
                        let _ = state_machine.transition(DownloadStatus::Cancelled);

                        if let Some(ref mut job) = *active_job_clone.write().await {
                            job.status = DownloadStatus::Cancelled;
                            job.completed_at = Some(Local::now().to_rfc3339());
                        }

                        break;
                    }

                    maybe_line = line_rx.recv() => {
                        match maybe_line {
                            Some(line) => {
                                diagnostics_clone.log("DEBUG", "YT_DLP", &line);

                                match parse_progress_line(&line) {
                                    ParsedLineEvent::Progress(prog) => {
                                        // 4 Hz coalescing per Extreme Performance Contract
                                        if last_progress_emit.elapsed() >= Duration::from_millis(250) {
                                            if let Some(ref mut job) = *active_job_clone.write().await {
                                                job.progress = prog;
                                            }
                                            last_progress_emit = Instant::now();
                                        }
                                    }
                                    ParsedLineEvent::PostProcessing(desc) => {
                                        diagnostics_clone.log("INFO", "POST_PROCESS", &desc);
                                        let _ = state_machine.transition(DownloadStatus::PostProcessing);
                                        if let Some(ref mut job) = *active_job_clone.write().await {
                                            job.status = DownloadStatus::PostProcessing;
                                            job.progress.raw_status_line = desc;
                                        }
                                    }
                                    ParsedLineEvent::Destination(dest) => {
                                        captured_destination = Some(dest);
                                    }
                                    ParsedLineEvent::Ignored => {}
                                }
                            }
                            None => {
                                // Channel closed, process finished output
                                break;
                            }
                        }
                    }
                }
            }

            if !is_cancelled {
                let success = proc_handle.wait_for_exit().await.unwrap_or(false);

                if success {
                    let _ = state_machine.transition(DownloadStatus::Verifying);
                    if let Some(ref mut job) = *active_job_clone.write().await {
                        job.status = DownloadStatus::Verifying;
                    }

                    // Resolve final file path
                    let final_path = if sidecar {
                        let filename =
                            if matches!(plan.operation, AcquisitionOperation::ThumbnailOnly) {
                                "artifact.jpg".to_string()
                            } else {
                                format!(
                                    "artifact.{}.vtt",
                                    plan.selected_streams.subtitle_languages[0]
                                )
                            };
                        let path = output_dir_path.join(filename);
                        path.is_file().then_some(path)
                    } else if let Some(dest) = captured_destination {
                        let p = PathBuf::from(dest);
                        if p.exists() {
                            Some(p)
                        } else {
                            resolve_final_download_path(&output_dir_path, &media_id)
                        }
                    } else {
                        resolve_final_download_path(&output_dir_path, &media_id)
                    };

                    match final_path {
                        Some(mut resolved_file) => {
                            if let Some(transform) = &compiled.finalize {
                                if let Some(job) = active_job_clone.write().await.as_mut() {
                                    job.status = DownloadStatus::PostProcessing;
                                }
                                let output = resolved_file.with_file_name(format!(
                                    "{}.converted.{}",
                                    resolved_file
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("media"),
                                    transform.container
                                ));
                                let result = match tool_resolver_clone
                                    .resolve_tool_with_settings(
                                        "ffmpeg",
                                        Some(&verification_settings),
                                    )
                                    .await
                                {
                                    Some(tool) => {
                                        run_final_transform(
                                            &tool.path,
                                            &transform.arguments(&resolved_file, &output),
                                            &mut cancel_rx,
                                            &diagnostics_clone,
                                        )
                                        .await
                                    }
                                    None => Err("Required FFmpeg became unavailable".into()),
                                };
                                if let Err(error) = result {
                                    if let Some(job) = active_job_clone.write().await.as_mut() {
                                        job.status = if error == "Cancelled" {
                                            DownloadStatus::Cancelled
                                        } else {
                                            DownloadStatus::Failed
                                        };
                                        job.error_message = Some(error);
                                        job.completed_at = Some(Local::now().to_rfc3339());
                                    }
                                    *active_handle_clone.lock().await = None;
                                    return;
                                }
                                resolved_file = output;
                                if let Some(job) = active_job_clone.write().await.as_mut() {
                                    job.status = DownloadStatus::Verifying;
                                }
                            }
                            diagnostics_clone.log(
                                "INFO",
                                "VERIFY",
                                &format!("Verifying media: {}", resolved_file.display()),
                            );

                            let fname = resolved_file
                                .file_name()
                                .and_then(|f| f.to_str())
                                .unwrap_or("media")
                                .to_string();

                            // Run media inspection through ffprobe/mediainfo
                            let ffprobe_tool = tool_resolver_clone
                                .resolve_tool_with_settings("ffprobe", Some(&verification_settings))
                                .await;
                            let inspection_result =
                                if matches!(plan.operation, AcquisitionOperation::SubtitlesOnly) {
                                    crate::media_verifier::verify_subtitle(&resolved_file).await
                                } else {
                                    verify_and_inspect_media(
                                        &resolved_file,
                                        ffprobe_tool.as_ref().map(|tool| tool.path.as_path()),
                                        is_lossy_warning,
                                    )
                                    .await
                                };

                            match inspection_result {
                                Ok(inspection) => {
                                    if let Some(ref mut job) = *active_job_clone.write().await {
                                        job.final_file_name = Some(fname);
                                        job.final_file_path =
                                            Some(resolved_file.to_string_lossy().to_string());
                                        job.inspection = Some(inspection.clone());
                                        job.progress.percentage = 100.0;
                                        job.completed_at = Some(Local::now().to_rfc3339());

                                        // 1. Generate Media Fingerprint
                                        let fp = crate::fingerprint::FingerprintEngine::generate(
                                            &job.metadata,
                                            Some(&inspection),
                                            None,
                                        );
                                        job.fingerprint = Some(fp);

                                        // 2. Generate Verification & Explainable Result
                                        let (verif, expl) = crate::media_verifier::generate_verification_and_explanation(
                                            job,
                                            &inspection,
                                            &resolved_file.to_string_lossy(),
                                        );
                                        job.verification = Some(verif);
                                        job.explainable_result = Some(expl);

                                        // 3. Create Reproducible Download Recipe
                                        let recipe = crate::recipe::RecipeEngine::create_recipe(
                                            job,
                                            job.verification.as_ref().map(|v| v.checklist.clone()),
                                        );
                                        if let Some(explanation) = job.explainable_result.as_mut() {
                                            explanation.recipe_id = Some(recipe.id.clone());
                                        }
                                        job.recipe = Some(recipe);

                                        if job.verification.as_ref().is_some_and(|v| v.is_valid) {
                                            let _ =
                                                state_machine.transition(DownloadStatus::Completed);
                                            job.status = DownloadStatus::Completed;
                                        } else {
                                            let _ =
                                                state_machine.transition(DownloadStatus::Failed);
                                            job.status = DownloadStatus::Failed;
                                            job.error_message = Some(
                                                "Deep media verification did not pass. The output was retained for inspection."
                                                    .to_string(),
                                            );
                                        }
                                    }
                                    let final_status = active_job_clone
                                        .read()
                                        .await
                                        .as_ref()
                                        .map(|job| job.status);
                                    diagnostics_clone.log(
                                        if final_status == Some(DownloadStatus::Completed) { "INFO" } else { "ERROR" },
                                        "DOWNLOAD_MANAGER",
                                        if final_status == Some(DownloadStatus::Completed) {
                                            "Download, verification, fingerprinting, and recipe generation completed successfully."
                                        } else {
                                            "Download finished, but deep media verification did not pass."
                                        },
                                    );
                                }
                                Err(verify_err) => {
                                    let _ = state_machine.transition(DownloadStatus::Failed);
                                    if let Some(ref mut job) = *active_job_clone.write().await {
                                        job.status = DownloadStatus::Failed;
                                        job.error_message =
                                            Some(format!("Verification failed: {}", verify_err));
                                        job.completed_at = Some(Local::now().to_rfc3339());
                                    }
                                }
                            }
                        }
                        None => {
                            let _ = state_machine.transition(DownloadStatus::Failed);
                            if let Some(ref mut job) = *active_job_clone.write().await {
                                job.status = DownloadStatus::Failed;
                                job.error_message = Some(
                                    "Verification failed: Could not locate output media file."
                                        .to_string(),
                                );
                                job.completed_at = Some(Local::now().to_rfc3339());
                            }
                        }
                    }
                } else {
                    let _ = state_machine.transition(DownloadStatus::Failed);
                    if let Some(ref mut job) = *active_job_clone.write().await {
                        job.status = DownloadStatus::Failed;
                        job.error_message =
                            Some("yt-dlp process returned a non-zero exit code.".to_string());
                        job.completed_at = Some(Local::now().to_rfc3339());
                    }
                }
            }

            *active_handle_clone.lock().await = None;
        });

        Ok(initial_job)
    }

    pub async fn cancel_download(&self, job_id: &str) -> Result<DownloadJob, String> {
        let cancellation_sent = {
            let handle_guard = self.active_handle.lock().await;
            if let Some(handle) = handle_guard.as_ref() {
                handle.job_id == job_id && handle.cancel_sender.send(()).await.is_ok()
            } else {
                false
            }
        };

        if !cancellation_sent {
            return Err(format!(
                "No running download job found with ID '{}'",
                job_id
            ));
        }

        if let Some(mut job) = self.get_active_job().await {
            if job.id == job_id {
                job.status = DownloadStatus::Cancelling;
                job.completed_at = None;
                *self.active_job.write().await = Some(job.clone());
                return Ok(job);
            }
        }

        Err(format!("No active download job found with ID '{}'", job_id))
    }
}

async fn run_final_transform(
    ffmpeg: &std::path::Path,
    args: &[String],
    cancel_rx: &mut mpsc::Receiver<()>,
    diagnostics: &DiagnosticsBuffer,
) -> Result<(), String> {
    if cancel_rx.try_recv().is_ok() {
        return Err("Cancelled".into());
    }
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut process = ProcessHandle::spawn_with_streaming(ffmpeg, args, tx).await?;
    loop {
        tokio::select! {
            _ = cancel_rx.recv() => { process.kill_tree().await; return Err("Cancelled".into()); }
            line = rx.recv() => match line {
                Some(line) => diagnostics.log("DEBUG", "FFMPEG", &line),
                None => break,
            }
        }
    }
    if process.wait_for_exit().await? {
        Ok(())
    } else {
        Err("Planned FFmpeg transformation failed; source retained.".into())
    }
}

fn required_tools(plan: &crate::types::AcquisitionPlan) -> Result<Vec<&'static str>, String> {
    plan.requirements
        .iter()
        .map(|requirement| match requirement.code.as_str() {
            "YT_DLP" => Ok("yt-dlp"),
            "FFMPEG" => Ok("ffmpeg"),
            "FFPROBE" => Ok("ffprobe"),
            code => Err(format!("Unsupported plan requirement: {code}")),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::tool_manager::ToolManager;
    use crate::types::{
        AcquisitionRequest, DuplicatePolicy, MediaFormatSpec, MediaKind, MediaMetadata,
        OutputProfile, SourceScope, TrackSelection,
    };

    fn test_job() -> DownloadJob {
        let metadata = MediaMetadata {
            id: "media".to_string(),
            title: "Media".to_string(),
            uploader: None,
            uploader_avatar: None,
            channel_id: None,
            uploader_url: None,
            duration: Some(1.0),
            thumbnail: None,
            webpage_url: "http://93.184.216.34/media".to_string(),
            media_kind: MediaKind::Video,
            upload_date: None,
            release_timestamp: None,
            view_count: None,
            like_count: None,
            description: None,
            categories: None,
            tags: None,
            language: None,
            is_live: Some(false),
            was_live: Some(false),
            extractor: None,
            extractor_key: None,
            playlist_title: None,
            playlist_index: None,
            playlist_count: None,
            available_resolutions: vec![720],
            available_frame_rates: vec![30],
            has_video: true,
            has_audio: true,
            is_hdr: None,
            subtitles: None,
            automatic_captions: None,
            chapters: None,
            formats: Some(vec![MediaFormatSpec {
                language: None,
                format_id: "combined".to_string(),
                ext: "mp4".to_string(),
                resolution: Some("1280x720".to_string()),
                width: Some(1280),
                height: Some(720),
                fps: Some(30.0),
                vcodec: Some("avc1.64001f".to_string()),
                acodec: Some("mp4a.40.2".to_string()),
                filesize: Some(1_000_000),
                filesize_approx: None,
                tbr: Some(1000.0),
                vbr: Some(900.0),
                abr: Some(128.0),
                hdr: Some(false),
                dynamic_range: None,
                audio_sample_rate: Some(48_000),
                audio_channels: Some(2),
            }]),
            smart_recommendation: None,
            source_type: None,
            strategy: None,
            transcoding_cost: None,
            transcoding_explanation: None,
            capabilities: None,
        };
        let acquisition = AcquisitionRequest {
            source_scope: SourceScope::SingleMedia,
            operation: AcquisitionOperation::EntireMedia,
            output_profile: OutputProfile::Universal,
            track_selection: TrackSelection::default(),
            metadata_patch: None,
            duplicate_policy: DuplicatePolicy::Rename,
            output_directory: "/tmp".to_string(),
            max_video_height: None,
        };
        let graph = crate::media_graph::MediaGraph::build_source_graph(
            &metadata,
            metadata.source_type.unwrap_or_default(),
        );
        let acquisition_plan = AcquisitionPlanner::plan(&graph, &acquisition).unwrap();

        DownloadJob {
            id: "job-cancel-test".to_string(),
            url: "https://example.com/media".to_string(),
            output_directory: "/tmp".to_string(),
            status: DownloadStatus::Downloading,
            progress: DownloadProgress::default(),
            metadata,
            final_file_name: None,
            final_file_path: None,
            inspection: None,
            error_message: None,
            created_at: "now".to_string(),
            completed_at: None,
            intent: None,
            recipe: None,
            fingerprint: None,
            explainable_result: None,
            verification: None,
            acquisition_plan,
        }
    }

    #[tokio::test]
    async fn cancel_reports_cancelling_until_worker_terminates_process() {
        let manager = DownloadManager::new(
            Arc::new(ToolResolver::new()),
            Arc::new(DiagnosticsBuffer::new()),
            Arc::new(SettingsManager::new()),
        );
        *manager.active_job.write().await = Some(test_job());
        let (sender, mut receiver) = mpsc::channel(1);
        *manager.active_handle.lock().await = Some(ActiveJobHandle {
            job_id: "job-cancel-test".to_string(),
            cancel_sender: sender,
        });

        let returned = manager.cancel_download("job-cancel-test").await.unwrap();
        assert_eq!(returned.status, DownloadStatus::Cancelling);
        assert!(returned.completed_at.is_none());
        assert_eq!(receiver.recv().await, Some(()));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn simultaneous_starts_reserve_only_one_download_slot() {
        let tools_directory = tempfile::tempdir().unwrap();
        let output_directory = tempfile::tempdir().unwrap();
        let diagnostics = Arc::new(DiagnosticsBuffer::new());
        let tool_manager = ToolManager::new(
            Some(tools_directory.path().to_path_buf()),
            diagnostics.clone(),
        );
        let fake_ytdlp = b"#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 2025.02.19; exit 0; fi\nsleep 5\nexit 1\n";
        let source = tools_directory.path().join("source");
        std::fs::write(&source, fake_ytdlp).unwrap();
        let hash = ToolManager::compute_sha256(&source).unwrap();
        tool_manager
            .install_from_bytes("yt-dlp", fake_ytdlp, &hash, false)
            .await
            .unwrap();

        let manager = Arc::new(DownloadManager::new(
            Arc::new(ToolResolver::with_manager(tool_manager)),
            diagnostics,
            Arc::new(SettingsManager::new()),
        ));
        let mut request = StartDownloadRequest {
            expected_plan_id: String::new(),
            metadata: test_job().metadata,
            acquisition: AcquisitionRequest {
                source_scope: SourceScope::SingleMedia,
                operation: AcquisitionOperation::EntireMedia,
                output_profile: OutputProfile::Universal,
                track_selection: TrackSelection::default(),
                metadata_patch: None,
                duplicate_policy: DuplicatePolicy::Rename,
                output_directory: output_directory.path().to_string_lossy().to_string(),
                max_video_height: None,
            },
        };

        let graph = crate::media_graph::MediaGraph::build_source_graph(
            &request.metadata,
            request.metadata.source_type.unwrap_or_default(),
        );
        request.expected_plan_id = AcquisitionPlanner::plan_with_policy(
            &graph,
            &request.acquisition,
            (&manager.settings.get_settings()).into(),
        )
        .unwrap()
        .id;
        let mut stale_request = request.clone();
        stale_request.expected_plan_id = "stale-plan".into();
        assert!(manager
            .start_download(stale_request)
            .await
            .unwrap_err()
            .contains("plan changed"));
        assert!(manager.get_active_job().await.is_none());
        let first_manager = manager.clone();
        let first_request = request.clone();
        let second_manager = manager.clone();
        let (first, second) = tokio::join!(
            async move { first_manager.start_download(first_request).await },
            async move { second_manager.start_download(request).await }
        );

        let started = match (first, second) {
            (Ok(started), Err(error)) | (Err(error), Ok(started)) => {
                assert!(error.contains("A download is already in progress"));
                started
            }
            results => panic!("expected exactly one admitted download, got {results:?}"),
        };
        let cancelled = manager.cancel_download(&started.id).await.unwrap();
        assert_eq!(cancelled.status, DownloadStatus::Cancelling);
        tokio::time::timeout(Duration::from_secs(2), async {
            loop {
                if manager
                    .get_active_job()
                    .await
                    .is_some_and(|job| job.status == DownloadStatus::Cancelled)
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
    }
}
