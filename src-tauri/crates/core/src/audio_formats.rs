pub fn is_aac(codec: &str) -> bool {
    let codec = codec.to_ascii_lowercase();
    codec.starts_with("mp4a") || codec == "aac"
}

/// Source-preserving extraction formats supported by the executor.
pub fn source_audio_container(
    codec: &str,
    source_container: Option<&str>,
) -> Result<&'static str, String> {
    if is_aac(codec) {
        return Ok(
            if source_container.is_some_and(|c| c.eq_ignore_ascii_case("aac")) {
                "aac"
            } else {
                "m4a"
            },
        );
    }
    match codec.to_ascii_lowercase().as_str() {
        "opus" => Ok("opus"),
        "vorbis" => Ok("ogg"),
        "flac" => Ok("flac"),
        "mp3" => Ok("mp3"),
        _ => Err(format!("Source-preserving audio extraction is not supported for codec '{codec}'; choose Universal or Editing")),
    }
}

pub fn extraction_format(container: &str) -> Result<&'static str, String> {
    match container {
        "aac" => Ok("aac"),
        "m4a" => Ok("m4a"),
        "opus" => Ok("opus"),
        "ogg" => Ok("vorbis"),
        "flac" => Ok("flac"),
        "mp3" => Ok("mp3"),
        _ => Err(format!(
            "Unsupported planned audio extraction container: {container}"
        )),
    }
}
