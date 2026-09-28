use crate::progress_parser::FINAL_PATH_PREFIX;
use crate::types::{
    AcquisitionOperation, AcquisitionPlan, AcquisitionRequest, AppSettings, OutputProfile,
    PresetType, ProcessingClass,
};

#[derive(Debug, Clone)]
pub struct CompiledPreset {
    pub arguments: Vec<String>,
    pub is_audio_only: bool,
    pub is_lossy_conversion: bool,
}

/// Compiles only the execution details already decided by the canonical planner.
/// Stream choice, output shape, and processing policy must not be inferred here.
pub fn compile_acquisition_args(
    plan: &AcquisitionPlan,
    request: &AcquisitionRequest,
    url: &str,
    settings: &AppSettings,
) -> Result<CompiledPreset, String> {
    if matches!(
        plan.operation,
        AcquisitionOperation::ThumbnailOnly | AcquisitionOperation::SubtitlesOnly
    ) {
        let mut args = vec![
            "--ignore-config".into(),
            "--no-playlist".into(),
            "--skip-download".into(),
            "--no-simulate".into(),
            "-o".into(),
            std::path::Path::new(&request.output_directory)
                .join("artifact.%(ext)s")
                .to_string_lossy()
                .into_owned(),
        ];
        if matches!(plan.operation, AcquisitionOperation::ThumbnailOnly) {
            args.extend([
                "--write-thumbnail".into(),
                "--convert-thumbnails".into(),
                "jpg".into(),
            ]);
        } else {
            args.extend([
                "--write-subs".into(),
                "--sub-format".into(),
                "vtt".into(),
                "--sub-langs".into(),
                plan.selected_streams.subtitle_languages.join(","),
            ]);
            if request.track_selection.include_auto_subtitles {
                args.push("--write-auto-subs".into());
            }
        }
        args.push(url.into());
        return Ok(CompiledPreset {
            arguments: args,
            is_audio_only: false,
            is_lossy_conversion: false,
        });
    }
    let mut args = vec![
        "--newline".to_string(),
        "--progress".to_string(),
        "--no-warnings".to_string(),
        "--print".to_string(),
        format!("after_move:{}%(filepath)s", FINAL_PATH_PREFIX),
        "--concurrent-fragments".to_string(),
        settings.concurrent_fragments.max(1).to_string(),
    ];

    let video_id = plan.selected_streams.video_stream_id.as_deref();
    let audio_id = plan.selected_streams.audio_stream_id.as_deref();
    let format_selector = match (video_id, audio_id) {
        (Some(video), Some(audio)) if video == audio => video.to_string(),
        (Some(video), Some(audio)) => format!("{video}+{audio}"),
        (Some(video), None) => video.to_string(),
        (None, Some(audio)) => audio.to_string(),
        (None, None) => {
            return Err("The acquisition plan selected no executable stream".to_string())
        }
    };
    args.push("-f".to_string());
    args.push(format_selector);
    args.push("--no-playlist".to_string());
    if let Some([start_ms, end_ms]) = plan.time_range_ms {
        args.extend([
            "--download-sections".to_string(),
            format!(
                "*{}.{:03}-{}.{:03}",
                start_ms / 1000,
                start_ms % 1000,
                end_ms / 1000,
                end_ms % 1000
            ),
        ]);
    }

    let is_audio_only = matches!(request.operation, AcquisitionOperation::AudioOnly);
    if is_audio_only {
        args.push("-x".to_string());
        if matches!(
            request.output_profile,
            OutputProfile::Universal | OutputProfile::Editing
        ) {
            args.push("--audio-format".to_string());
            args.push(plan.output.container.clone());
            if request.output_profile == OutputProfile::Universal {
                args.push("--audio-quality".to_string());
                args.push("0".to_string());
            }
        }
    } else {
        args.push("--merge-output-format".to_string());
        args.push(plan.output.container.clone());
        if matches!(
            plan.processing.class,
            ProcessingClass::AudioTranscode
                | ProcessingClass::VideoTranscode
                | ProcessingClass::FullTranscode
        ) {
            args.push("--recode-video".to_string());
            args.push(plan.output.container.clone());
        }
    }

    if settings.embed_metadata {
        args.push("--embed-metadata".to_string());
    }
    if settings.embed_thumbnail {
        args.push("--embed-thumbnail".to_string());
    }
    if settings.embed_chapters && !is_audio_only {
        args.push("--embed-chapters".to_string());
    }
    match settings.sponsor_block_mode {
        crate::types::SponsorBlockMode::MarkChapters => {
            args.extend(["--sponsorblock-mark".to_string(), "all".to_string()]);
        }
        crate::types::SponsorBlockMode::RemoveSegments => {
            args.extend(["--sponsorblock-remove".to_string(), "all".to_string()]);
        }
        crate::types::SponsorBlockMode::Off => {}
    }

    if !plan.selected_streams.subtitle_languages.is_empty() && !is_audio_only {
        args.push("--sub-langs".to_string());
        args.push(plan.selected_streams.subtitle_languages.join(","));
        if matches!(settings.subtitle_mode, crate::types::SubtitleMode::Embed) {
            args.push("--embed-subs".to_string());
        } else {
            args.push("--write-subs".to_string());
        }
    }

    let separator =
        if request.output_directory.ends_with('/') || request.output_directory.ends_with('\\') {
            ""
        } else if cfg!(windows) || request.output_directory.contains('\\') {
            "\\"
        } else {
            "/"
        };
    args.push("-o".to_string());
    let range_suffix = match plan.operation {
        AcquisitionOperation::Chapter { chapter_index } => {
            format!(" [chapter-{}]", chapter_index + 1)
        }
        AcquisitionOperation::Clip { start_ms, end_ms } => format!(" [clip-{start_ms}-{end_ms}]"),
        _ => String::new(),
    };
    args.push(format!(
        "{}{}%(title).{}B [%(id)s]{}.%(ext)s",
        request.output_directory,
        separator,
        settings.trim_filenames.max(1),
        range_suffix
    ));
    args.push(url.to_string());

    Ok(CompiledPreset {
        arguments: args,
        is_audio_only,
        is_lossy_conversion: matches!(
            plan.processing.class,
            ProcessingClass::AudioTranscode
                | ProcessingClass::VideoTranscode
                | ProcessingClass::FullTranscode
        ),
    })
}

pub fn compile_download_args(
    preset: PresetType,
    quality: &str,
    output_dir: &str,
    url: &str,
    settings: &AppSettings,
) -> CompiledPreset {
    let mut args: Vec<String> = Vec::new();

    let is_audio = matches!(
        preset,
        PresetType::BestAudio | PresetType::Mp3 | PresetType::Flac
    );
    let is_lossy_conversion = preset == PresetType::Mp3;

    // Progress & stdout stream configuration
    args.push("--newline".to_string());
    args.push("--progress".to_string());
    args.push("--no-warnings".to_string());
    args.push("--print".to_string());
    args.push(format!("after_move:{}%(filepath)s", FINAL_PATH_PREFIX));

    // Performance contract: 1 concurrent fragment to prevent stalls
    let fragments = if settings.concurrent_fragments > 0 {
        settings.concurrent_fragments.to_string()
    } else {
        "1".to_string()
    };
    args.push("--concurrent-fragments".to_string());
    args.push(fragments);

    let max_title_bytes = if settings.trim_filenames > 0 {
        settings.trim_filenames
    } else {
        180
    };

    // Format selection & Preset specific flags
    match preset {
        PresetType::Mp4Compatible => {
            let f = if quality != "auto" && !quality.is_empty() {
                format!(
                    "bv*[ext=mp4][height<={q}]+ba[ext=m4a]/b[ext=mp4][height<={q}]/bv*[height<={q}]+ba/b[height<={q}]",
                    q = quality
                )
            } else {
                "bv*[ext=mp4]+ba[ext=m4a]/b[ext=mp4]/bv+ba/b".to_string()
            };
            args.push("-f".to_string());
            args.push(f.clone());
            args.push("--merge-output-format".to_string());
            args.push("mp4".to_string());
        }
        PresetType::BestVideo => {
            let f = if quality != "auto" && !quality.is_empty() {
                format!("bv*[height<={q}]+ba/b[height<={q}]/bv+ba/b", q = quality)
            } else {
                "bv+ba/b".to_string()
            };
            args.push("-f".to_string());
            args.push(f.clone());
            args.push("--merge-output-format".to_string());
            args.push("mkv".to_string());
        }
        PresetType::BestAudio => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
        }
        PresetType::Mp3 => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
            args.push("--audio-quality".to_string());
            args.push("0".to_string());
        }
        PresetType::Flac => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("flac".to_string());
        }
    }

    // Metadata & Chapter flags
    if settings.embed_metadata {
        args.push("--embed-metadata".to_string());
    }
    if settings.embed_thumbnail {
        args.push("--embed-thumbnail".to_string());
    }
    if settings.embed_chapters && !is_audio {
        args.push("--embed-chapters".to_string());
    }

    // SponsorBlock support
    match settings.sponsor_block_mode {
        crate::types::SponsorBlockMode::MarkChapters => {
            args.push("--sponsorblock-mark".to_string());
            args.push("all".to_string());
        }
        crate::types::SponsorBlockMode::RemoveSegments => {
            args.push("--sponsorblock-remove".to_string());
            args.push("all".to_string());
        }
        crate::types::SponsorBlockMode::Off => {}
    }

    // Subtitle support
    match settings.subtitle_mode {
        crate::types::SubtitleMode::Embed if !is_audio => {
            args.push("--embed-subs".to_string());
            args.push("--sub-langs".to_string());
            let lang = if settings.preferred_subtitle_language.is_empty() {
                "en.*,en".to_string()
            } else {
                settings.preferred_subtitle_language.clone()
            };
            args.push(lang);
        }
        crate::types::SubtitleMode::DownloadSeparate => {
            args.push("--write-subs".to_string());
            args.push("--sub-langs".to_string());
            let lang = if settings.preferred_subtitle_language.is_empty() {
                "en.*,en".to_string()
            } else {
                settings.preferred_subtitle_language.clone()
            };
            args.push(lang);
        }
        _ => {}
    }

    // Output template
    let sep = if output_dir.ends_with('/') || output_dir.ends_with('\\') {
        ""
    } else if cfg!(windows) || output_dir.contains('\\') {
        "\\"
    } else {
        "/"
    };

    let template = format!(
        "{}{}%(title).{}B [%(id)s].%(ext)s",
        output_dir, sep, max_title_bytes
    );
    args.push("-o".to_string());
    args.push(template);

    // Target URL
    args.push(url.to_string());

    CompiledPreset {
        arguments: args,
        is_audio_only: is_audio,
        is_lossy_conversion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_arg_pair(arguments: &[String], flag: &str, value: &str) -> bool {
        arguments
            .windows(2)
            .any(|pair| pair[0] == flag && pair[1] == value)
    }

    #[test]
    fn test_mp4_compatible_preset() {
        let settings = AppSettings::default();
        let compiled = compile_download_args(
            PresetType::Mp4Compatible,
            "1080",
            "/tmp/downloads",
            "https://example.com/video",
            &settings,
        );

        assert!(compiled.arguments.contains(&"-f".to_string()));
        assert!(compiled
            .arguments
            .iter()
            .any(|a| a.contains("height<=1080")));
        assert!(compiled
            .arguments
            .contains(&"--merge-output-format".to_string()));
        assert!(has_arg_pair(
            &compiled.arguments,
            "--merge-output-format",
            "mp4"
        ));
        assert!(!compiled.is_audio_only);
        assert!(has_arg_pair(
            &compiled.arguments,
            "--print",
            "after_move:__OCMD_FINAL_PATH__%(filepath)s"
        ));
    }

    #[test]
    fn test_best_video_preset() {
        let settings = AppSettings::default();
        let compiled = compile_download_args(
            PresetType::BestVideo,
            "auto",
            "/tmp/downloads",
            "https://example.com/video",
            &settings,
        );

        assert!(has_arg_pair(&compiled.arguments, "-f", "bv+ba/b"));
        assert!(has_arg_pair(
            &compiled.arguments,
            "--merge-output-format",
            "mkv"
        ));
        assert!(!compiled.is_audio_only);
    }

    #[test]
    fn test_best_audio_preserves_source_format() {
        let settings = AppSettings::default();
        let compiled = compile_download_args(
            PresetType::BestAudio,
            "auto",
            "/tmp/downloads",
            "https://example.com/audio",
            &settings,
        );

        assert!(compiled.arguments.contains(&"-x".to_string()));
        assert!(has_arg_pair(&compiled.arguments, "-f", "bestaudio/b"));
        assert!(!compiled.arguments.contains(&"--audio-format".to_string()));
        assert!(compiled.is_audio_only);
        assert!(!compiled.is_lossy_conversion);
    }

    #[test]
    fn test_mp3_preset() {
        let settings = AppSettings::default();
        let compiled = compile_download_args(
            PresetType::Mp3,
            "auto",
            "/tmp/downloads",
            "https://example.com/audio",
            &settings,
        );

        assert!(compiled.arguments.contains(&"-x".to_string()));
        assert!(has_arg_pair(&compiled.arguments, "--audio-format", "mp3"));
        assert!(has_arg_pair(&compiled.arguments, "--audio-quality", "0"));
        assert!(compiled.is_audio_only);
        assert!(compiled.is_lossy_conversion);
    }

    #[test]
    fn test_flac_is_not_marked_as_lossy_conversion() {
        let settings = AppSettings::default();
        let compiled = compile_download_args(
            PresetType::Flac,
            "auto",
            "/tmp/downloads",
            "https://example.com/audio",
            &settings,
        );

        assert!(!compiled.is_lossy_conversion);
        assert!(has_arg_pair(&compiled.arguments, "--audio-format", "flac"));
    }

    #[test]
    fn test_sponsorblock_and_subtitles() {
        let settings = AppSettings {
            sponsor_block_mode: crate::types::SponsorBlockMode::MarkChapters,
            subtitle_mode: crate::types::SubtitleMode::Embed,
            preferred_subtitle_language: "en,es".to_string(),
            ..AppSettings::default()
        };

        let compiled = compile_download_args(
            PresetType::Mp4Compatible,
            "1080",
            "/tmp/downloads",
            "https://example.com/video",
            &settings,
        );

        assert!(compiled
            .arguments
            .contains(&"--sponsorblock-mark".to_string()));
        assert!(compiled.arguments.contains(&"--embed-subs".to_string()));
        assert!(compiled.arguments.contains(&"en,es".to_string()));
        assert!(compiled.arguments.contains(&"--embed-chapters".to_string()));
    }
}
