use crate::types::{AppSettings, PresetType};

#[derive(Debug, Clone)]
pub struct CompiledPreset {
    pub arguments: Vec<String>,
    pub output_extension: String,
    pub is_audio_only: bool,
    pub is_lossy_conversion: bool,
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
    let (fmt_string, expected_ext) = match preset {
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
            (f, "mp4".to_string())
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
            (f, "mkv".to_string())
        }
        PresetType::BestAudio => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
            ("bestaudio/b".to_string(), "m4a".to_string())
        }
        PresetType::Mp3 => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("mp3".to_string());
            args.push("--audio-quality".to_string());
            args.push("0".to_string());
            ("bestaudio/b".to_string(), "mp3".to_string())
        }
        PresetType::Flac => {
            args.push("-f".to_string());
            args.push("bestaudio/b".to_string());
            args.push("-x".to_string());
            args.push("--audio-format".to_string());
            args.push("flac".to_string());
            ("bestaudio/b".to_string(), "flac".to_string())
        }
    };

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
        output_extension: expected_ext,
        is_audio_only: is_audio,
        is_lossy_conversion,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(compiled.arguments.iter().any(|a| a.contains("height<=1080")));
        assert!(compiled.arguments.contains(&"--merge-output-format".to_string()));
        assert!(compiled.arguments.contains(&"mp4".to_string()));
        assert!(!compiled.is_audio_only);
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
        assert!(compiled.arguments.contains(&"--audio-format".to_string()));
        assert!(compiled.arguments.contains(&"mp3".to_string()));
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
        assert!(compiled.arguments.contains(&"flac".to_string()));
    }

    #[test]
    fn test_sponsorblock_and_subtitles() {
        let mut settings = AppSettings::default();
        settings.sponsor_block_mode = crate::types::SponsorBlockMode::MarkChapters;
        settings.subtitle_mode = crate::types::SubtitleMode::Embed;
        settings.preferred_subtitle_language = "en,es".to_string();

        let compiled = compile_download_args(
            PresetType::Mp4Compatible,
            "1080",
            "/tmp/downloads",
            "https://example.com/video",
            &settings,
        );

        assert!(compiled.arguments.contains(&"--sponsorblock-mark".to_string()));
        assert!(compiled.arguments.contains(&"--embed-subs".to_string()));
        assert!(compiled.arguments.contains(&"en,es".to_string()));
        assert!(compiled.arguments.contains(&"--embed-chapters".to_string()));
    }
}
