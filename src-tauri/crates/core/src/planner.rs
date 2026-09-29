use sha2::{Digest, Sha256};

use crate::types::{
    AcquisitionOperation, AcquisitionPlan, AcquisitionRequest, AudioStreamSpec, OutputProfile,
    PlanRequirement, PlanWarning, PlannedArtifact, PostProcessPolicy, ProcessingClass,
    ProcessingPlan, SelectedStreams, SizeEstimate, SourceMediaGraph, SourceSummary,
    VideoStreamSpec,
};

pub const PLAN_VERSION: u32 = 3;

pub struct AcquisitionPlanner;

impl AcquisitionPlanner {
    pub fn plan(
        source: &SourceMediaGraph,
        request: &AcquisitionRequest,
    ) -> Result<AcquisitionPlan, String> {
        let mut plan = Self::plan_base(source, request)?;
        use crate::types::PlannedTransform as T;
        let class = plan.processing.class;
        plan.transforms.clear();
        if plan.selected_streams.video_stream_id.is_some()
            && plan.selected_streams.audio_stream_id.is_some()
            && plan.selected_streams.video_stream_id != plan.selected_streams.audio_stream_id
        {
            plan.transforms.push(T::Merge);
        }
        if plan.output.audio_only {
            plan.transforms.push(T::ExtractAudio {
                format: plan.output.container.clone(),
            });
        }
        if class == ProcessingClass::RemuxOnly {
            plan.transforms.push(T::Remux {
                container: plan.output.container.clone(),
            });
        }
        if matches!(
            class,
            ProcessingClass::VideoTranscode | ProcessingClass::FullTranscode
        ) {
            plan.transforms.push(T::TranscodeVideo {
                codec: plan
                    .output
                    .video_codec
                    .clone()
                    .ok_or("Missing planned video codec")?,
            });
        }
        if matches!(
            class,
            ProcessingClass::AudioTranscode | ProcessingClass::FullTranscode
        ) {
            plan.transforms.push(T::TranscodeAudio {
                codec: plan
                    .output
                    .audio_codec
                    .clone()
                    .ok_or("Missing planned audio codec")?,
            });
        }
        if let Some([start_ms, end_ms]) = plan.time_range_ms {
            plan.transforms.push(T::Trim { start_ms, end_ms });
        }
        Ok(plan)
    }

    pub fn plan_with_policy(
        source: &SourceMediaGraph,
        request: &AcquisitionRequest,
        policy: PostProcessPolicy,
    ) -> Result<AcquisitionPlan, String> {
        let mut effective_request = request.clone();
        let use_preference = matches!(
            request.operation,
            AcquisitionOperation::EntireMedia
                | AcquisitionOperation::Clip { .. }
                | AcquisitionOperation::Chapter { .. }
        ) && policy.subtitle_mode != crate::types::SubtitleMode::None
            && request.track_selection.subtitle_languages.is_empty();
        if use_preference {
            effective_request.track_selection.subtitle_languages =
                crate::subtitle_selection::resolve_subtitle_preference(
                    &policy.preferred_subtitle_language,
                    &source.subtitle_streams,
                    request.track_selection.include_auto_subtitles,
                )?;
        }
        let mut plan = Self::plan(source, &effective_request)?;
        if use_preference && plan.selected_streams.subtitle_languages.is_empty() {
            plan.warnings.push(PlanWarning { code: "SUBTITLE_PREFERENCE_UNAVAILABLE".into(), message: format!("No available subtitle tracks match preference {:?}; no subtitles will be downloaded.", policy.preferred_subtitle_language) });
        }
        let sidecar = matches!(
            plan.operation,
            AcquisitionOperation::ThumbnailOnly | AcquisitionOperation::SubtitlesOnly
        );
        if !sidecar {
            plan.post_process = policy;
            let policy = &plan.post_process;
            for (enabled, step) in [
                (policy.embed_metadata, "Embed source metadata"),
                (policy.embed_thumbnail, "Embed cover artwork"),
                (
                    policy.embed_chapters && !plan.output.audio_only,
                    "Embed chapters",
                ),
                (
                    policy.subtitle_mode == crate::types::SubtitleMode::Embed
                        && !plan.selected_streams.subtitle_languages.is_empty(),
                    "Embed selected subtitles",
                ),
                (
                    policy.sponsor_block_mode != crate::types::SponsorBlockMode::Off,
                    "Apply SponsorBlock policy",
                ),
            ] {
                if enabled {
                    plan.processing.steps.push(step.into());
                    plan.processing.requires_ffmpeg = true;
                }
            }
            if policy.subtitle_mode == crate::types::SubtitleMode::DownloadSeparate
                && !plan.selected_streams.subtitle_languages.is_empty()
                && !plan.output.audio_only
            {
                plan.processing
                    .steps
                    .push("Save selected subtitles alongside the media".into());
            }
            if policy.sponsor_block_mode == crate::types::SponsorBlockMode::RemoveSegments {
                plan.warnings.push(PlanWarning { code: "CONTENT_REMOVAL".into(), message: "SponsorBlock removes source segments. Output duration is content-dependent.".into() });
            }
        }
        plan.requirements = vec![PlanRequirement {
            code: "YT_DLP".into(),
            message: "yt-dlp is required for acquisition.".into(),
        }];
        if plan.processing.requires_ffmpeg {
            plan.requirements.push(PlanRequirement {
                code: "FFMPEG".into(),
                message: "FFmpeg is required for the planned processing.".into(),
            });
        }
        if !matches!(plan.operation, AcquisitionOperation::SubtitlesOnly) {
            plan.requirements.push(PlanRequirement {
                code: "FFPROBE".into(),
                message: "FFprobe is required for artifact verification.".into(),
            });
        }
        let bytes = serde_json::to_vec(&plan).map_err(|e| format!("Cannot serialize plan: {e}"))?;
        plan.id = format!(
            "plan-v{PLAN_VERSION}-{}",
            &hex::encode(Sha256::digest(bytes))[..20]
        );
        Ok(plan)
    }

    fn plan_base(
        source: &SourceMediaGraph,
        request: &AcquisitionRequest,
    ) -> Result<AcquisitionPlan, String> {
        validate_request(source, request)?;
        if let AcquisitionOperation::Chapter { chapter_index } = request.operation {
            let chapter = &source.chapters[chapter_index as usize];
            if !chapter.start_time.is_finite()
                || !chapter.end_time.is_finite()
                || chapter.start_time < 0.0
                || chapter.end_time <= chapter.start_time
            {
                return Err("Chapter has invalid time boundaries".into());
            }
            let mut clip_request = request.clone();
            clip_request.operation = AcquisitionOperation::Clip {
                start_ms: (chapter.start_time * 1000.0).round() as u64,
                end_ms: (chapter.end_time * 1000.0).round() as u64,
            };
            let mut plan = Self::plan(source, &clip_request)?;
            plan.operation = request.operation.clone();
            plan.id = stable_plan_id(source, request)?;
            return Ok(plan);
        }
        if matches!(
            request.operation,
            AcquisitionOperation::ThumbnailOnly | AcquisitionOperation::SubtitlesOnly
        ) {
            let thumbnail = matches!(request.operation, AcquisitionOperation::ThumbnailOnly);
            return Ok(AcquisitionPlan {
                transforms: vec![],
                include_auto_subtitles: request.track_selection.include_auto_subtitles,
                post_process: PostProcessPolicy::default(),
                time_range_ms: None,
                id: stable_plan_id(source, request)?,
                version: PLAN_VERSION,
                source: SourceSummary {
                    url: source.source_url.clone(),
                    title: source.title.clone(),
                    extractor: source.extractor.clone(),
                    media_kind: source.media_kind,
                    duration_seconds: source.duration_seconds,
                },
                scope: request.source_scope,
                operation: request.operation.clone(),
                output_profile: request.output_profile,
                selected_streams: SelectedStreams {
                    subtitle_languages: request.track_selection.subtitle_languages.clone(),
                    ..SelectedStreams::default()
                },
                output: PlannedArtifact {
                    container: if thumbnail { "jpg" } else { "vtt" }.into(),
                    video_codec: None,
                    audio_codec: None,
                    width: None,
                    height: None,
                    fps: None,
                    audio_only: false,
                },
                processing: ProcessingPlan {
                    class: ProcessingClass::Unknown,
                    requires_ffmpeg: thumbnail,
                    steps: vec![if thumbnail {
                        "Save thumbnail as JPEG"
                    } else {
                        "Save selected WebVTT subtitle track"
                    }
                    .into()],
                },
                estimated_size: None,
                warnings: vec![],
                requirements: vec![PlanRequirement {
                    code: "YT_DLP".into(),
                    message: "yt-dlp required; thumbnail conversion also needs FFmpeg.".into(),
                }],
            });
        }

        let audio_only = matches!(request.operation, AcquisitionOperation::AudioOnly);
        let mut selected_video = if audio_only {
            None
        } else {
            select_video(source, request, false)
        };
        let mut selected_audio = if !audio_only
            && selected_video.is_some_and(|video| format_has_audio(source, &video.stream_id))
        {
            selected_video.and_then(|video| {
                source
                    .audio_streams
                    .iter()
                    .find(|audio| audio.stream_id == video.stream_id)
            })
        } else {
            select_audio(source, request, !audio_only)
        };

        let combined_fallback =
            !audio_only && selected_audio.is_none() && !source.audio_streams.is_empty();
        if combined_fallback {
            selected_video = select_video(source, request, true);
            selected_audio = selected_video.and_then(|video| {
                source
                    .audio_streams
                    .iter()
                    .find(|audio| audio.stream_id == video.stream_id)
            });
        }

        if let Some(wanted) = &request.track_selection.audio_language {
            if selected_audio.and_then(|a| a.language.as_ref()) != Some(wanted) {
                return Err(format!(
                    "Selected format cannot supply requested audio language: {wanted}"
                ));
            }
        }
        if !audio_only && selected_video.is_none() {
            return Err("The source has no usable video stream for this operation".to_string());
        }
        if selected_audio.is_none() && (audio_only || !source.audio_streams.is_empty()) {
            return Err("The source has no usable audio stream for this operation".to_string());
        }

        let (container, video_codec, audio_codec, processing_class) =
            planned_output(source, request, selected_video, selected_audio);
        let requires_ffmpeg = processing_class != ProcessingClass::SourcePreserved
            || matches!(request.operation, AcquisitionOperation::Clip { .. })
            || audio_only
            || selected_video
                .zip(selected_audio)
                .is_some_and(|(video, audio)| video.stream_id != audio.stream_id);

        let mut warnings = Vec::new();
        if combined_fallback {
            warnings.push(PlanWarning { code: "COMBINED_SOURCE_FALLBACK".into(), message: "No suitable audio-only format is available; use one combined video/audio source.".into() });
        }
        if matches!(request.operation, AcquisitionOperation::Clip { .. }) {
            warnings.push(PlanWarning {
                code: "FAST_CUT_BOUNDARIES".to_string(),
                message: "Fast cuts may align to nearby keyframes; the actual duration can differ from the requested range.".to_string(),
            });
        }
        if matches!(
            processing_class,
            ProcessingClass::AudioTranscode
                | ProcessingClass::VideoTranscode
                | ProcessingClass::FullTranscode
        ) {
            warnings.push(PlanWarning {
                code: "LOSSY_TRANSCODE".to_string(),
                message: "The requested profile requires re-encoding one or more source streams."
                    .to_string(),
            });
        }
        if request
            .max_video_height
            .is_some_and(|limit| selected_video.is_some_and(|video| video.height < limit))
        {
            warnings.push(PlanWarning {
                code: "SOURCE_BELOW_REQUESTED_HEIGHT".to_string(),
                message: "The source does not contain a stream at the requested height."
                    .to_string(),
            });
        }
        if request
            .max_video_height
            .is_some_and(|limit| selected_video.is_some_and(|video| video.height > limit))
        {
            warnings.push(PlanWarning {
                code: "HEIGHT_LIMIT_UNAVAILABLE".to_string(),
                message: "No source stream satisfies the requested maximum height; the smallest available stream was selected."
                    .to_string(),
            });
        }

        let mut requirements = vec![PlanRequirement {
            code: "YT_DLP".to_string(),
            message: "yt-dlp is required to acquire the selected source streams.".to_string(),
        }];
        if requires_ffmpeg {
            requirements.push(PlanRequirement {
                code: "FFMPEG".to_string(),
                message: "FFmpeg is required for the planned merge or conversion.".to_string(),
            });
        }

        let steps = processing_steps(processing_class);
        let selected_streams = SelectedStreams {
            video_stream_id: selected_video.map(|stream| stream.stream_id.clone()),
            audio_stream_id: selected_audio.map(|stream| stream.stream_id.clone()),
            subtitle_languages: request.track_selection.subtitle_languages.clone(),
        };
        let estimated_size = if matches!(request.operation, AcquisitionOperation::Clip { .. }) {
            None
        } else {
            estimate_size(selected_video, selected_audio)
        };
        let output = PlannedArtifact {
            container,
            video_codec,
            audio_codec,
            width: selected_video.map(|stream| stream.width),
            height: selected_video.map(|stream| stream.height),
            fps: selected_video.and_then(|stream| stream.fps),
            audio_only,
        };

        Ok(AcquisitionPlan {
            transforms: vec![],
            include_auto_subtitles: request.track_selection.include_auto_subtitles,
            post_process: PostProcessPolicy::default(),
            time_range_ms: match request.operation {
                AcquisitionOperation::Clip { start_ms, end_ms } => Some([start_ms, end_ms]),
                _ => None,
            },
            id: stable_plan_id(source, request)?,
            version: PLAN_VERSION,
            source: SourceSummary {
                url: source.source_url.clone(),
                title: source.title.clone(),
                extractor: source.extractor.clone(),
                media_kind: source.media_kind,
                duration_seconds: source.duration_seconds,
            },
            scope: request.source_scope,
            operation: request.operation.clone(),
            output_profile: request.output_profile,
            selected_streams,
            output,
            processing: ProcessingPlan {
                class: processing_class,
                requires_ffmpeg,
                steps,
            },
            estimated_size,
            warnings,
            requirements,
        })
    }
}

fn validate_request(source: &SourceMediaGraph, request: &AcquisitionRequest) -> Result<(), String> {
    if request.metadata_patch.is_some() {
        return Err("Metadata patching is not implemented for this slice".into());
    }
    if request.duplicate_policy != crate::types::DuplicatePolicy::Rename {
        return Err("Only Rename duplicate policy is supported".into());
    }
    if request.output_profile == OutputProfile::Custom {
        return Err("Custom output profiles are not implemented".into());
    }
    for language in &request.track_selection.subtitle_languages {
        if !language
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            || !source.subtitle_streams.iter().any(|track| {
                track.language == *language
                    && (request.track_selection.include_auto_subtitles
                        || !track.is_auto.unwrap_or(false))
            })
        {
            return Err(format!(
                "Selected subtitle language is unavailable: {language}"
            ));
        }
    }
    if let Some(language) = &request.track_selection.audio_language {
        if !source
            .audio_streams
            .iter()
            .any(|s| s.language.as_ref() == Some(language))
        {
            return Err(format!(
                "Requested audio language is unavailable: {language}"
            ));
        }
    }
    if request.output_directory.trim().is_empty() {
        return Err("An output directory is required".to_string());
    }
    match request.operation {
        AcquisitionOperation::EntireMedia | AcquisitionOperation::AudioOnly => Ok(()),
        AcquisitionOperation::ThumbnailOnly => {
            if source.thumbnails.is_empty() {
                Err("No thumbnail is available".into())
            } else {
                Ok(())
            }
        }
        AcquisitionOperation::SubtitlesOnly => {
            let languages = &request.track_selection.subtitle_languages;
            if languages.len() != 1 {
                return Err("Select exactly one subtitle language per download".into());
            }
            let language = &languages[0];
            if !language
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                || !source.subtitle_streams.iter().any(|track| {
                    track.language == *language
                        && (request.track_selection.include_auto_subtitles
                            || !track.is_auto.unwrap_or(false))
                })
            {
                return Err("The selected subtitle language is unavailable".into());
            }
            Ok(())
        }
        AcquisitionOperation::Clip { start_ms, end_ms } if start_ms >= end_ms => {
            Err("Clip end must be after clip start".to_string())
        }
        AcquisitionOperation::Clip { end_ms, .. } => {
            let duration = source
                .duration_seconds
                .filter(|d| d.is_finite() && *d > 0.0)
                .ok_or("Clip requires a source with a known finite duration")?;
            if end_ms as f64 > duration * 1000.0 {
                return Err("Clip end exceeds the source duration".to_string());
            }
            Ok(())
        }
        AcquisitionOperation::Chapter { chapter_index }
            if chapter_index as usize >= source.chapters.len() =>
        {
            Err("The selected chapter does not exist".to_string())
        }
        AcquisitionOperation::Chapter { .. } => Ok(()),
    }
}

fn select_video<'a>(
    source: &'a SourceMediaGraph,
    request: &AcquisitionRequest,
    combined_only: bool,
) -> Option<&'a VideoStreamSpec> {
    let available = || {
        source.video_streams.iter().filter(|video| {
            let combined = format_has_audio(source, &video.stream_id);
            (!combined_only || combined)
                && (!combined
                    || request
                        .track_selection
                        .audio_language
                        .as_ref()
                        .is_none_or(|wanted| {
                            source.audio_streams.iter().any(|a| {
                                a.stream_id == video.stream_id
                                    && a.language.as_ref() == Some(wanted)
                            })
                        }))
        })
    };
    let eligible = || {
        available().filter(|stream| {
            request
                .max_video_height
                .is_none_or(|max_height| stream.height <= max_height)
        })
    };

    let compatible = eligible()
        .filter(|stream| is_h264(&stream.codec))
        .max_by(video_quality_cmp);

    match request.output_profile {
        OutputProfile::Universal | OutputProfile::Editing => {
            compatible.or_else(|| eligible().max_by(video_quality_cmp))
        }
        OutputProfile::Small => {
            eligible().min_by_key(|stream| (stream.height, stream.bitrate_kbps.unwrap_or(u64::MAX)))
        }
        OutputProfile::BestSource | OutputProfile::Custom => eligible().max_by(video_quality_cmp),
    }
    .or_else(|| available().min_by_key(|stream| stream.height))
}

fn video_quality_cmp(left: &&VideoStreamSpec, right: &&VideoStreamSpec) -> std::cmp::Ordering {
    left.height
        .cmp(&right.height)
        .then_with(|| left.width.cmp(&right.width))
        .then_with(|| left.fps.unwrap_or(0.0).total_cmp(&right.fps.unwrap_or(0.0)))
        .then_with(|| left.bitrate_kbps.cmp(&right.bitrate_kbps))
}

fn select_audio<'a>(
    source: &'a SourceMediaGraph,
    request: &AcquisitionRequest,
    standalone_only: bool,
) -> Option<&'a AudioStreamSpec> {
    let language_matches = |stream: &&AudioStreamSpec| {
        request
            .track_selection
            .audio_language
            .as_ref()
            .is_none_or(|wanted| stream.language.as_ref() == Some(wanted))
    };
    let candidates = || {
        source
            .audio_streams
            .iter()
            .filter(|s| !standalone_only || s.is_audio_only)
            .filter(language_matches)
    };
    let compatible = candidates()
        .filter(|stream| is_aac(&stream.codec))
        .max_by_key(|stream| stream.bitrate_kbps.unwrap_or(0));

    match request.output_profile {
        OutputProfile::Universal | OutputProfile::Editing
            if !matches!(request.operation, AcquisitionOperation::AudioOnly) =>
        {
            compatible
                .or_else(|| candidates().max_by_key(|stream| stream.bitrate_kbps.unwrap_or(0)))
        }
        OutputProfile::Small => {
            candidates().min_by_key(|stream| stream.bitrate_kbps.unwrap_or(u64::MAX))
        }
        _ => candidates().max_by_key(|stream| stream.bitrate_kbps.unwrap_or(0)),
    }
}

fn planned_output(
    source: &SourceMediaGraph,
    request: &AcquisitionRequest,
    video: Option<&VideoStreamSpec>,
    audio: Option<&AudioStreamSpec>,
) -> (String, Option<String>, Option<String>, ProcessingClass) {
    let audio_only = matches!(request.operation, AcquisitionOperation::AudioOnly);
    if audio_only {
        return match request.output_profile {
            OutputProfile::Universal => (
                "mp3".to_string(),
                None,
                Some("mp3".to_string()),
                if audio.is_some_and(|stream| stream.codec.eq_ignore_ascii_case("mp3")) {
                    ProcessingClass::SourcePreserved
                } else {
                    ProcessingClass::AudioTranscode
                },
            ),
            OutputProfile::Editing => (
                "flac".to_string(),
                None,
                Some("flac".to_string()),
                if audio.is_some_and(|stream| stream.codec.eq_ignore_ascii_case("flac")) {
                    ProcessingClass::SourcePreserved
                } else {
                    ProcessingClass::AudioTranscode
                },
            ),
            _ => {
                let codec = audio.map(|stream| stream.codec.clone());
                let container = audio_container(codec.as_deref());
                (container, None, codec, ProcessingClass::SourcePreserved)
            }
        };
    }

    let source_video_codec = video.map(|stream| stream.codec.clone());
    let source_audio_codec = audio.map(|stream| stream.codec.clone());
    match request.output_profile {
        OutputProfile::Universal | OutputProfile::Editing => {
            let video_transcode = video.is_some_and(|stream| !is_h264(&stream.codec));
            let audio_transcode = audio.is_some_and(|stream| !is_aac(&stream.codec));
            let class = match (video_transcode, audio_transcode) {
                (true, true) => ProcessingClass::FullTranscode,
                (true, false) => ProcessingClass::VideoTranscode,
                (false, true) => ProcessingClass::AudioTranscode,
                (false, false) if streams_need_merge(video, audio) => ProcessingClass::MergeOnly,
                (false, false)
                    if video
                        .and_then(|v| format_ext(source, &v.stream_id))
                        .as_deref()
                        != Some("mp4") =>
                {
                    ProcessingClass::RemuxOnly
                }
                _ => ProcessingClass::SourcePreserved,
            };
            (
                "mp4".to_string(),
                video.map(|_| "h264".to_string()),
                audio.map(|_| "aac".to_string()),
                class,
            )
        }
        _ => {
            let needs_merge = streams_need_merge(video, audio);
            let container = if needs_merge {
                "mkv".to_string()
            } else {
                video
                    .and_then(|stream| format_ext(source, &stream.stream_id))
                    .unwrap_or_else(|| "mkv".to_string())
            };
            (
                container,
                source_video_codec,
                source_audio_codec,
                if needs_merge {
                    ProcessingClass::MergeOnly
                } else {
                    ProcessingClass::SourcePreserved
                },
            )
        }
    }
}

fn streams_need_merge(video: Option<&VideoStreamSpec>, audio: Option<&AudioStreamSpec>) -> bool {
    video
        .zip(audio)
        .is_some_and(|(video, audio)| video.stream_id != audio.stream_id)
}

fn format_ext(source: &SourceMediaGraph, stream_id: &str) -> Option<String> {
    source
        .formats
        .iter()
        .find(|format| format.format_id == stream_id)
        .map(|format| format.ext.clone())
}

fn format_has_audio(source: &SourceMediaGraph, stream_id: &str) -> bool {
    source.formats.iter().any(|format| {
        format.format_id == stream_id
            && format
                .acodec
                .as_deref()
                .is_some_and(|codec| !codec.eq_ignore_ascii_case("none"))
    })
}

fn audio_container(codec: Option<&str>) -> String {
    match codec.unwrap_or_default().to_ascii_lowercase().as_str() {
        "opus" => "opus",
        "vorbis" => "ogg",
        "aac" | "mp4a" | "mp4a.40.2" => "m4a",
        "flac" => "flac",
        "mp3" => "mp3",
        _ => "mka",
    }
    .to_string()
}

fn is_h264(codec: &str) -> bool {
    let codec = codec.to_ascii_lowercase();
    codec.starts_with("avc1") || codec == "h264"
}

fn is_aac(codec: &str) -> bool {
    let codec = codec.to_ascii_lowercase();
    codec.starts_with("mp4a") || codec == "aac"
}

fn processing_steps(class: ProcessingClass) -> Vec<String> {
    match class {
        ProcessingClass::SourcePreserved => vec!["Preserve selected source streams".to_string()],
        ProcessingClass::MergeOnly => {
            vec!["Merge selected streams without re-encoding".to_string()]
        }
        ProcessingClass::RemuxOnly => vec!["Remux streams into the output container".to_string()],
        ProcessingClass::AudioTranscode => {
            vec!["Transcode audio for the output profile".to_string()]
        }
        ProcessingClass::VideoTranscode => {
            vec!["Transcode video for the output profile".to_string()]
        }
        ProcessingClass::FullTranscode => {
            vec!["Transcode video and audio for the output profile".to_string()]
        }
        ProcessingClass::Unknown => vec!["Processing could not be classified".to_string()],
    }
}

fn estimate_size(
    video: Option<&VideoStreamSpec>,
    audio: Option<&AudioStreamSpec>,
) -> Option<SizeEstimate> {
    let mut ids = Vec::new();
    let mut bytes = 0_u64;
    for (id, size) in [
        video.map(|stream| (&stream.stream_id, stream.filesize_approx)),
        audio.map(|stream| (&stream.stream_id, stream.filesize_approx)),
    ]
    .into_iter()
    .flatten()
    {
        if !ids.contains(&id.as_str()) {
            bytes = bytes.checked_add(size?)?;
            ids.push(id.as_str());
        }
    }
    (bytes > 0).then(|| SizeEstimate {
        bytes,
        confidence: "SOURCE_REPORTED".to_string(),
    })
}

fn stable_plan_id(
    source: &SourceMediaGraph,
    request: &AcquisitionRequest,
) -> Result<String, String> {
    let mut hasher = Sha256::new();
    hasher.update(format!("acquisition-plan-v{PLAN_VERSION}:"));
    hasher.update(serde_json::to_vec(source).map_err(|e| format!("Cannot serialize source: {e}"))?);
    hasher.update(b":");
    hasher
        .update(serde_json::to_vec(request).map_err(|e| format!("Cannot serialize request: {e}"))?);
    let digest = hex::encode(hasher.finalize());
    Ok(format!("plan-v{PLAN_VERSION}-{}", &digest[..20]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        DuplicatePolicy, MediaFormatSpec, MediaKind, MediaSourceType, SourceScope, TrackSelection,
    };

    fn source_graph() -> SourceMediaGraph {
        SourceMediaGraph {
            source_url: "https://example.com/watch/1".to_string(),
            extractor: "test".to_string(),
            source_type: MediaSourceType::YtDlpExtractor,
            title: "Example".to_string(),
            media_kind: MediaKind::Video,
            duration_seconds: Some(60.0),
            video_streams: vec![
                VideoStreamSpec {
                    stream_id: "vp9-1440".to_string(),
                    codec: "vp9".to_string(),
                    profile: None,
                    width: 2560,
                    height: 1440,
                    fps: Some(60.0),
                    bitrate_kbps: Some(6000),
                    is_hdr: false,
                    dynamic_range: None,
                    aspect_ratio: None,
                    filesize_approx: Some(45_000_000),
                },
                VideoStreamSpec {
                    stream_id: "h264-1080".to_string(),
                    codec: "avc1.640028".to_string(),
                    profile: None,
                    width: 1920,
                    height: 1080,
                    fps: Some(30.0),
                    bitrate_kbps: Some(3500),
                    is_hdr: false,
                    dynamic_range: None,
                    aspect_ratio: None,
                    filesize_approx: Some(28_000_000),
                },
            ],
            audio_streams: vec![AudioStreamSpec {
                is_audio_only: true,
                stream_id: "aac-128".to_string(),
                codec: "mp4a.40.2".to_string(),
                bitrate_kbps: Some(128),
                sample_rate_hz: Some(48_000),
                channels: Some(2),
                language: None,
                is_default: true,
                filesize_approx: Some(1_000_000),
            }],
            subtitle_streams: vec![],
            chapters: vec![],
            thumbnails: vec![],
            formats: vec![MediaFormatSpec {
                format_id: "aac-128".to_string(),
                ext: "m4a".to_string(),
                ..MediaFormatSpec::default()
            }],
        }
    }

    fn request(profile: OutputProfile, operation: AcquisitionOperation) -> AcquisitionRequest {
        AcquisitionRequest {
            source_scope: SourceScope::SingleMedia,
            operation,
            output_profile: profile,
            track_selection: TrackSelection::default(),
            metadata_patch: None,
            duplicate_policy: DuplicatePolicy::Rename,
            output_directory: "/tmp/downloads".to_string(),
            max_video_height: None,
        }
    }

    #[test]
    fn universal_prefers_compatible_streams_without_claiming_transcode() {
        let acquisition = request(OutputProfile::Universal, AcquisitionOperation::EntireMedia);
        let plan = AcquisitionPlanner::plan(&source_graph(), &acquisition).unwrap();
        assert_eq!(
            plan.selected_streams.video_stream_id.as_deref(),
            Some("h264-1080")
        );
        assert_eq!(plan.output.container, "mp4");
        assert_eq!(plan.processing.class, ProcessingClass::MergeOnly);

        let compiled = crate::execution::compile_acquisition_args(
            &plan,
            &crate::execution::ExecutionContext::new(
                &acquisition.output_directory,
                &crate::types::AppSettings::default(),
            ),
        )
        .unwrap();
        assert!(compiled
            .arguments
            .windows(2)
            .any(|pair| pair == ["-f", "h264-1080+aac-128"]));
    }

    #[test]
    fn best_source_selects_highest_quality() {
        let plan = AcquisitionPlanner::plan(
            &source_graph(),
            &request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia),
        )
        .unwrap();
        assert_eq!(
            plan.selected_streams.video_stream_id.as_deref(),
            Some("vp9-1440")
        );
        assert_eq!(plan.output.video_codec.as_deref(), Some("vp9"));
    }

    #[test]
    fn plan_identity_is_stable_and_request_sensitive() {
        let source = source_graph();
        let first_request = request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia);
        let first = AcquisitionPlanner::plan(&source, &first_request).unwrap();
        let repeated = AcquisitionPlanner::plan(&source, &first_request).unwrap();
        let different = AcquisitionPlanner::plan(
            &source,
            &request(OutputProfile::Universal, AcquisitionOperation::EntireMedia),
        )
        .unwrap();
        assert_eq!(first.id, repeated.id);
        assert_ne!(first.id, different.id);
    }

    #[test]
    fn audio_universal_is_truthfully_classified_as_transcode() {
        let plan = AcquisitionPlanner::plan(
            &source_graph(),
            &request(OutputProfile::Universal, AcquisitionOperation::AudioOnly),
        )
        .unwrap();
        assert_eq!(plan.output.container, "mp3");
        assert_eq!(plan.processing.class, ProcessingClass::AudioTranscode);
        assert_eq!(
            plan.selected_streams.audio_stream_id.as_deref(),
            Some("aac-128")
        );
    }

    #[test]
    fn executable_operation_profile_matrix_stays_authoritative() {
        let source = source_graph();
        let settings = crate::types::AppSettings::default();
        for operation in [
            AcquisitionOperation::EntireMedia,
            AcquisitionOperation::AudioOnly,
        ] {
            for profile in [
                OutputProfile::BestSource,
                OutputProfile::Universal,
                OutputProfile::Editing,
                OutputProfile::Small,
            ] {
                let acquisition = request(profile, operation.clone());
                let plan = AcquisitionPlanner::plan(&source, &acquisition).unwrap();
                assert_eq!(plan.operation, operation);
                assert_eq!(plan.output_profile, profile);

                let compiled = crate::execution::compile_acquisition_args(
                    &plan,
                    &crate::execution::ExecutionContext::new(
                        &acquisition.output_directory,
                        &settings,
                    ),
                )
                .unwrap();
                assert!(!compiled.arguments.is_empty());
            }
        }
    }

    #[test]
    fn clip_validates_bounds_and_compiles_exact_range() {
        let source = source_graph();
        let mut acquisition = request(
            OutputProfile::BestSource,
            AcquisitionOperation::Clip {
                start_ms: 1000,
                end_ms: 2500,
            },
        );
        let plan = AcquisitionPlanner::plan(&source, &acquisition).unwrap();
        assert!(plan.processing.requires_ffmpeg);
        assert!(plan.estimated_size.is_none());
        let compiled = crate::execution::compile_acquisition_args(
            &plan,
            &crate::execution::ExecutionContext::new(
                &acquisition.output_directory,
                &crate::types::AppSettings::default(),
            ),
        )
        .unwrap();
        assert!(compiled
            .arguments
            .windows(2)
            .any(|pair| pair == ["--download-sections", "*1.000-2.500"]));
        assert!(compiled
            .arguments
            .iter()
            .any(|arg| arg.contains("[clip-1000-2500]")));
        acquisition.operation = AcquisitionOperation::Clip {
            start_ms: 1000,
            end_ms: 1000,
        };
        assert!(AcquisitionPlanner::plan(&source, &acquisition).is_err());
        acquisition.operation = AcquisitionOperation::Clip {
            start_ms: 0,
            end_ms: u64::MAX,
        };
        assert!(AcquisitionPlanner::plan(&source, &acquisition).is_err());
    }

    #[test]
    fn sidecar_and_chapter_plans_compile_without_media_download() {
        let mut source = source_graph();
        source.thumbnails.push(crate::types::ThumbnailSpec {
            url: "https://example.com/image.jpg".into(),
            width: None,
            height: None,
            id: None,
        });
        source.subtitle_streams.push(crate::types::SubtitleTrack {
            language: "en".into(),
            is_auto: Some(true),
            ..Default::default()
        });
        source.chapters.push(crate::types::MediaChapter {
            title: "Intro".into(),
            start_time: 1.0,
            end_time: 3.0,
        });
        let settings = crate::types::AppSettings::default();
        let chapter = request(
            OutputProfile::BestSource,
            AcquisitionOperation::Chapter { chapter_index: 0 },
        );
        let plan = AcquisitionPlanner::plan(&source, &chapter).unwrap();
        assert_eq!(plan.time_range_ms, Some([1000, 3000]));
        assert!(matches!(
            plan.operation,
            AcquisitionOperation::Chapter { .. }
        ));
        for operation in [
            AcquisitionOperation::ThumbnailOnly,
            AcquisitionOperation::SubtitlesOnly,
        ] {
            let mut acquisition = request(OutputProfile::BestSource, operation);
            if matches!(acquisition.operation, AcquisitionOperation::SubtitlesOnly) {
                acquisition.track_selection.subtitle_languages = vec!["en".into()];
                assert!(AcquisitionPlanner::plan(&source, &acquisition).is_err());
                acquisition.track_selection.include_auto_subtitles = true;
            }
            let plan = AcquisitionPlanner::plan(&source, &acquisition).unwrap();
            let compiled = crate::execution::compile_acquisition_args(
                &plan,
                &crate::execution::ExecutionContext::new(&acquisition.output_directory, &settings),
            )
            .unwrap();
            assert!(compiled
                .arguments
                .iter()
                .any(|arg| arg == "--skip-download"));
            assert!(!compiled
                .arguments
                .iter()
                .any(|arg| arg == "-f" || arg == "--embed-thumbnail"));
        }
    }

    #[test]
    fn policy_changes_identity_and_compiler_uses_reviewed_policy() {
        let source = source_graph();
        let request = request(OutputProfile::BestSource, AcquisitionOperation::EntireMedia);
        let first =
            AcquisitionPlanner::plan_with_policy(&source, &request, PostProcessPolicy::default())
                .unwrap();
        let policy = PostProcessPolicy {
            embed_metadata: true,
            sponsor_block_mode: crate::types::SponsorBlockMode::RemoveSegments,
            ..Default::default()
        };
        let changed = AcquisitionPlanner::plan_with_policy(&source, &request, policy).unwrap();
        assert_ne!(first.id, changed.id);
        assert!(changed.warnings.iter().any(|w| w.code == "CONTENT_REMOVAL"));
        assert!(changed.requirements.iter().any(|r| r.code == "FFPROBE"));
        let args = crate::execution::compile_acquisition_args(
            &first,
            &crate::execution::ExecutionContext::new(
                &request.output_directory,
                &crate::types::AppSettings::default(),
            ),
        )
        .unwrap()
        .arguments;
        assert!(!args
            .iter()
            .any(|a| a == "--embed-thumbnail" || a == "--embed-metadata"));
    }

    #[test]
    fn small_video_profile_selects_the_smallest_suitable_stream() {
        let plan = AcquisitionPlanner::plan(
            &source_graph(),
            &request(OutputProfile::Small, AcquisitionOperation::EntireMedia),
        )
        .unwrap();
        assert_eq!(
            plan.selected_streams.video_stream_id.as_deref(),
            Some("h264-1080")
        );
        assert_eq!(plan.output_profile, OutputProfile::Small);
    }
}
