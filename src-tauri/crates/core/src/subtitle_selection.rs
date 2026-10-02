use crate::types::SubtitleTrack;
use regex::RegexBuilder;
use std::collections::BTreeSet;

/// Resolve settings syntax here; the compiler receives only concrete language tags.
pub fn resolve_subtitle_preference(
    preference: &str,
    tracks: &[SubtitleTrack],
    include_auto: bool,
) -> Result<Vec<String>, String> {
    let preference = if preference.trim().is_empty() {
        "en"
    } else {
        preference.trim()
    };
    if preference.len() > 2048 {
        return Err("Subtitle language preference exceeds 2048 characters".into());
    }
    let mut include = Vec::new();
    let mut exclude = Vec::new();
    for token in preference.split(',').map(str::trim) {
        let (negative, pattern) = token
            .strip_prefix('-')
            .map_or((false, token), |p| (true, p));
        if pattern.is_empty() {
            return Err("Subtitle language preference contains an empty pattern".into());
        }
        let pattern = if pattern == "all" { ".*" } else { pattern };
        let regex = RegexBuilder::new(&format!("^(?:{pattern})$"))
            .size_limit(256 * 1024)
            .build()
            .map_err(|e| format!("Unsupported subtitle language pattern '{pattern}': {e}"))?;
        if negative {
            exclude.push(regex);
        } else {
            include.push(regex);
        }
    }
    Ok(tracks
        .iter()
        .filter(|track| include_auto || !track.is_auto.unwrap_or(false))
        .map(|track| &track.language)
        .filter(|language| {
            include.iter().any(|re| re.is_match(language))
                && !exclude.iter().any(|re| re.is_match(language))
        })
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect())
}
