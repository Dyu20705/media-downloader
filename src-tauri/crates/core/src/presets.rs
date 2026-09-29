use crate::progress_parser::FINAL_PATH_PREFIX;
use crate::types::{
    AcquisitionOperation, AcquisitionPlan, AppSettings, PlannedTransform, SponsorBlockMode,
    SubtitleMode,
};
use std::path::Path;

pub struct ExecutionContext {
    pub output_directory: String,
    pub concurrent_fragments: u32,
    pub trim_filenames: u32,
}

impl ExecutionContext {
    pub fn new(output_directory: &str, settings: &AppSettings) -> Self {
        Self {
            output_directory: output_directory.into(),
            concurrent_fragments: settings.concurrent_fragments.max(1),
            trim_filenames: settings.trim_filenames.max(1),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompiledAcquisition {
    pub arguments: Vec<String>,
    pub is_lossy_conversion: bool,
    pub finalize: Option<FinalTransform>,
}

#[derive(Debug, Clone)]
pub struct FinalTransform {
    pub container: String,
    pub video_encoder: Option<String>,
    pub audio_encoder: Option<String>,
}

impl FinalTransform {
    pub fn arguments(&self, input: &Path, output: &Path) -> Vec<String> {
        let mut args = vec![
            "-nostdin".into(),
            "-n".into(),
            "-i".into(),
            input.to_string_lossy().into_owned(),
            "-map".into(),
            "0".into(),
            "-map_metadata".into(),
            "0".into(),
            "-map_chapters".into(),
            "0".into(),
            "-c".into(),
            "copy".into(),
        ];
        if let Some(codec) = &self.video_encoder {
            args.extend(["-c:v:0".into(), codec.clone()]);
        }
        if let Some(codec) = &self.audio_encoder {
            args.extend(["-c:a:0".into(), codec.clone()]);
        }
        args.push(output.to_string_lossy().into_owned());
        args
    }
}

/// Only operational parameters are supplied outside the reviewed plan.
pub fn compile_acquisition_args(
    plan: &AcquisitionPlan,
    context: &ExecutionContext,
) -> Result<CompiledAcquisition, String> {
    let mut args = vec![
        "--ignore-config".into(),
        "--no-playlist".into(),
        "--no-simulate".into(),
        "--newline".into(),
    ];
    let thumbnail = matches!(plan.operation, AcquisitionOperation::ThumbnailOnly);
    let subtitle = matches!(plan.operation, AcquisitionOperation::SubtitlesOnly);
    if thumbnail || subtitle {
        args.extend([
            "--skip-download".into(),
            "-o".into(),
            Path::new(&context.output_directory)
                .join("artifact.%(ext)s")
                .to_string_lossy()
                .into_owned(),
        ]);
        if thumbnail {
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
            if plan.include_auto_subtitles {
                args.push("--write-auto-subs".into());
            }
        }
        args.push(plan.source.url.clone());
        return Ok(CompiledAcquisition {
            arguments: args,
            is_lossy_conversion: false,
            finalize: None,
        });
    }
    let selector = match (
        plan.selected_streams.video_stream_id.as_deref(),
        plan.selected_streams.audio_stream_id.as_deref(),
    ) {
        (Some(v), Some(a)) if v != a => format!("{v}+{a}"),
        (Some(v), _) => v.into(),
        (_, Some(a)) => a.into(),
        _ => return Err("Plan has no executable streams".into()),
    };
    args.extend([
        "--progress".into(),
        "--print".into(),
        format!("after_move:{}%(filepath)s", FINAL_PATH_PREFIX),
        "--concurrent-fragments".into(),
        context.concurrent_fragments.to_string(),
        "-f".into(),
        selector,
    ]);
    let mut finalize = FinalTransform {
        container: plan.output.container.clone(),
        video_encoder: None,
        audio_encoder: None,
    };
    let mut lossy = false;
    for transform in &plan.transforms {
        match transform {
            PlannedTransform::Merge => {
                args.extend([
                    "--merge-output-format".into(),
                    if plan.output.audio_only {
                        plan.output.container.clone()
                    } else {
                        "mkv".into()
                    },
                ]);
            }
            PlannedTransform::Remux { container } => {
                args.extend(["--remux-video".into(), container.clone()])
            }
            PlannedTransform::ExtractAudio { format } => {
                let encoder = match format.as_str() {
                    "ogg" => "vorbis",
                    "mka" => {
                        return Err("Source audio codec has no supported extraction format".into())
                    }
                    other => other,
                };
                args.extend(["-x".into(), "--audio-format".into(), encoder.into()]);
                if format == "mp3" {
                    args.extend(["--audio-quality".into(), "0".into()]);
                }
            }
            PlannedTransform::TranscodeVideo { codec } => {
                finalize.video_encoder = Some(match codec.as_str() {
                    "h264" => "libx264".into(),
                    _ => return Err(format!("Unsupported planned video codec: {codec}")),
                });
                lossy = true;
            }
            PlannedTransform::TranscodeAudio { codec } => {
                lossy |= codec != "flac";
                if !plan.output.audio_only {
                    finalize.audio_encoder = Some(match codec.as_str() {
                        "aac" => "aac".into(),
                        _ => return Err(format!("Unsupported planned audio codec: {codec}")),
                    });
                }
            }
            PlannedTransform::Trim { start_ms, end_ms } => args.extend([
                "--download-sections".into(),
                format!(
                    "*{}.{:03}-{}.{:03}",
                    start_ms / 1000,
                    start_ms % 1000,
                    end_ms / 1000,
                    end_ms % 1000
                ),
            ]),
        }
    }
    let finalize = if finalize.video_encoder.is_some() || finalize.audio_encoder.is_some() {
        Some(finalize)
    } else {
        None
    };
    if finalize.is_none()
        && plan
            .transforms
            .iter()
            .any(|t| matches!(t, PlannedTransform::Merge))
    {
        // This is the planned mux target; no codec choices are made here.
        if let Some(index) = args.iter().position(|a| a == "--merge-output-format") {
            args[index + 1] = plan.output.container.clone();
        }
    }
    let policy = &plan.post_process;
    if policy.embed_metadata {
        args.push("--embed-metadata".into());
    }
    if policy.embed_thumbnail {
        args.push("--embed-thumbnail".into());
    }
    if policy.embed_chapters && !plan.output.audio_only {
        args.push("--embed-chapters".into());
    }
    match policy.sponsor_block_mode {
        SponsorBlockMode::MarkChapters => args.extend(["--sponsorblock-mark".into(), "all".into()]),
        SponsorBlockMode::RemoveSegments => {
            args.extend(["--sponsorblock-remove".into(), "all".into()])
        }
        SponsorBlockMode::Off => {}
    }
    if !plan.selected_streams.subtitle_languages.is_empty()
        && !plan.output.audio_only
        && policy.subtitle_mode != SubtitleMode::None
    {
        args.extend([
            "--sub-langs".into(),
            plan.selected_streams.subtitle_languages.join(","),
        ]);
        args.push(
            if policy.subtitle_mode == SubtitleMode::Embed {
                "--embed-subs"
            } else {
                "--write-subs"
            }
            .into(),
        );
        if plan.include_auto_subtitles {
            args.push("--write-auto-subs".into());
        }
    }
    let suffix = match plan.operation {
        AcquisitionOperation::Clip { start_ms, end_ms } => format!(" [clip-{start_ms}-{end_ms}]"),
        AcquisitionOperation::Chapter { chapter_index } => {
            format!(" [chapter-{}]", chapter_index + 1)
        }
        _ => String::new(),
    };
    args.extend([
        "-o".into(),
        Path::new(&context.output_directory)
            .join(format!(
                "%(title).{}B [%(id)s]{suffix}.%(ext)s",
                context.trim_filenames
            ))
            .to_string_lossy()
            .into_owned(),
        plan.source.url.clone(),
    ]);
    Ok(CompiledAcquisition {
        arguments: args,
        is_lossy_conversion: lossy,
        finalize,
    })
}
