use chrono::Local;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex, RwLock};

use crate::diagnostics::DiagnosticsBuffer;
use crate::media_verifier::{resolve_final_download_path, verify_and_inspect_media};
use crate::path_validator::validate_and_ensure_directory;
use crate::presets::compile_download_args;
use crate::process_runner::ProcessHandle;
use crate::progress_parser::{parse_progress_line, ParsedLineEvent};
use crate::settings::SettingsManager;
use crate::state_machine::DownloadStateMachine;
use crate::tools::ToolResolver;
use crate::types::{DownloadJob, DownloadProgress, DownloadStatus, StartDownloadRequest};
use crate::url_validator::validate_media_url;

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
        }
    }

    pub async fn get_active_job(&self) -> Option<DownloadJob> {
        self.active_job.read().await.clone()
    }

    pub async fn start_download(
        &self,
        request: StartDownloadRequest,
    ) -> Result<DownloadJob, String> {
        // 1. Check concurrency lock (1 active job per performance contract)
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

        // 2. Validate URL and Output Directory
        let valid_url = validate_media_url(&request.url).map_err(|e| e.to_string())?;
        let output_dir_path =
            validate_and_ensure_directory(&request.output_directory).map_err(|e| e.to_string())?;

        // 3. Resolve yt-dlp tool
        let ytdlp_tool = self
            .tool_resolver
            .resolve_tool("yt-dlp")
            .await
            .ok_or_else(|| "yt-dlp executable not found".to_string())?;

        let current_settings = self.settings.get_settings();
        let compiled = compile_download_args(
            request.preset,
            &request.quality,
            &output_dir_path.to_string_lossy(),
            &valid_url,
            &current_settings,
        );

        let job_id = format!("job-{}", Local::now().timestamp_millis());
        let now_str = Local::now().to_rfc3339();

        let initial_job = DownloadJob {
            id: job_id.clone(),
            url: valid_url.clone(),
            preset: request.preset,
            quality: request.quality.clone(),
            output_directory: output_dir_path.to_string_lossy().to_string(),
            status: DownloadStatus::Downloading,
            progress: DownloadProgress::default(),
            metadata: request.metadata.clone(),
            final_file_name: None,
            final_file_path: None,
            inspection: None,
            verification: None,
            error_message: None,
            created_at: now_str,
            completed_at: None,
            subtitle_options: None,
            sponsor_block_mode: Some(current_settings.sponsor_block_mode),
            intent: None,
            recipe: None,
            fingerprint: None,
            explainable_result: None,
        };

        // Store initial job in state
        *self.active_job.write().await = Some(initial_job.clone());

        self.diagnostics.log(
            "INFO",
            "DOWNLOAD_MANAGER",
            &format!(
                "Starting download for '{}' using preset {:?}",
                request.metadata.title, request.preset
            ),
        );

        let (line_tx, mut line_rx) = mpsc::unbounded_channel::<String>();
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

        let mut final_args = compiled.arguments.clone();
        if let Some(ffmpeg_tool) = self.tool_resolver.resolve_tool("ffmpeg").await {
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
                    let final_path = if let Some(dest) = captured_destination {
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
                        Some(resolved_file) => {
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
                            let inspection_result = verify_and_inspect_media(
                                &resolved_file,
                                &tool_resolver_clone,
                                is_lossy_warning,
                            )
                            .await;

                            match inspection_result {
                                Ok(inspection) => {
                                    let _ = state_machine.transition(DownloadStatus::Completed);
                                    if let Some(ref mut job) = *active_job_clone.write().await {
                                        job.status = DownloadStatus::Completed;
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
                                        job.recipe = Some(recipe);
                                    }
                                    diagnostics_clone.log(
                                        "INFO",
                                        "DOWNLOAD_MANAGER",
                                        "Download, verification, fingerprinting, and recipe generation completed successfully.",
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
        let handle_guard = self.active_handle.lock().await;
        if let Some(handle) = handle_guard.as_ref() {
            if handle.job_id == job_id {
                let _ = handle.cancel_sender.send(()).await;
            }
        }

        if let Some(mut job) = self.get_active_job().await {
            if job.id == job_id {
                job.status = DownloadStatus::Cancelled;
                job.completed_at = Some(Local::now().to_rfc3339());
                *self.active_job.write().await = Some(job.clone());
                return Ok(job);
            }
        }

        Err(format!("No active download job found with ID '{}'", job_id))
    }
}
