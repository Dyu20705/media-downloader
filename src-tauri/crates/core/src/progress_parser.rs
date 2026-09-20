use crate::types::DownloadProgress;
use regex::Regex;
use std::sync::OnceLock;

static DOWNLOAD_REGEX: OnceLock<Regex> = OnceLock::new();

fn get_download_regex() -> &'static Regex {
    DOWNLOAD_REGEX.get_or_init(|| {
        Regex::new(r"\[download\]\s+([0-9\.]+)%\s+of\s+(?:~\s*)?([0-9\.]+\s*[a-zA-Z]+)(?:\s+at\s+([0-9\.]+\s*[a-zA-Z]+/s))?(?:\s+ETA\s+([0-9:]+))?").unwrap()
    })
}

pub enum ParsedLineEvent {
    Progress(DownloadProgress),
    PostProcessing(String),
    Destination(String),
    Ignored,
}

pub fn parse_progress_line(line: &str) -> ParsedLineEvent {
    let trimmed = line.trim();

    if trimmed.starts_with("[download]") {
        let re = get_download_regex();
        if let Some(caps) = re.captures(trimmed) {
            let pct: f64 = caps
                .get(1)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0.0);
            let total_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let speed_str = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let eta_str = caps.get(4).map(|m| m.as_str()).unwrap_or("");

            let total_bytes = parse_size_bytes(total_str);
            let downloaded_bytes = ((pct / 100.0) * (total_bytes as f64)) as u64;
            let speed_bytes = parse_speed_bytes(speed_str);
            let eta_seconds = parse_eta_seconds(eta_str);

            return ParsedLineEvent::Progress(DownloadProgress {
                percentage: pct,
                downloaded_bytes,
                total_bytes,
                speed_bytes_per_sec: speed_bytes,
                eta_seconds,
                current_speed: if speed_str.is_empty() {
                    "--".to_string()
                } else {
                    speed_str.to_string()
                },
                raw_status_line: trimmed.to_string(),
            });
        }
    } else if trimmed.starts_with("[Merger]")
        || trimmed.starts_with("[ExtractAudio]")
        || trimmed.starts_with("[FixupM3u8]")
        || trimmed.starts_with("[EmbedSubtitle]")
    {
        return ParsedLineEvent::PostProcessing(trimmed.to_string());
    } else if trimmed.starts_with("[download] Destination:") {
        let dest = trimmed.trim_start_matches("[download] Destination:").trim();
        return ParsedLineEvent::Destination(dest.to_string());
    }

    ParsedLineEvent::Ignored
}

fn parse_size_bytes(size_str: &str) -> u64 {
    let trimmed = size_str.trim().to_uppercase();
    if trimmed.is_empty() {
        return 0;
    }

    let mut num_str = String::new();
    let mut unit_str = String::new();

    for c in trimmed.chars() {
        if c.is_ascii_digit() || c == '.' {
            num_str.push(c);
        } else if c.is_ascii_alphabetic() {
            unit_str.push(c);
        }
    }

    let val: f64 = num_str.parse().unwrap_or(0.0);
    let multiplier: f64 = match unit_str.as_str() {
        "KIB" | "KB" | "K" => 1024.0,
        "MIB" | "MB" | "M" => 1024.0 * 1024.0,
        "GIB" | "GB" | "G" => 1024.0 * 1024.0 * 1024.0,
        "TIB" | "TB" | "T" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };

    (val * multiplier) as u64
}

fn parse_speed_bytes(speed_str: &str) -> f64 {
    let trimmed = speed_str.trim().to_uppercase().replace("/S", "");
    parse_size_bytes(&trimmed) as f64
}

fn parse_eta_seconds(eta_str: &str) -> Option<u64> {
    let parts: Vec<&str> = eta_str.split(':').collect();
    match parts.len() {
        2 => {
            let mins: u64 = parts[0].parse().ok()?;
            let secs: u64 = parts[1].parse().ok()?;
            Some(mins * 60 + secs)
        }
        3 => {
            let hrs: u64 = parts[0].parse().ok()?;
            let mins: u64 = parts[1].parse().ok()?;
            let secs: u64 = parts[2].parse().ok()?;
            Some(hrs * 3600 + mins * 60 + secs)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ytdlp_progress_line() {
        let line = "[download]  45.2% of ~ 100.00MiB at   5.00MiB/s ETA 00:11";
        if let ParsedLineEvent::Progress(prog) = parse_progress_line(line) {
            assert_eq!(prog.percentage, 45.2);
            assert_eq!(prog.total_bytes, 100 * 1024 * 1024);
            assert_eq!(prog.eta_seconds, Some(11));
            assert_eq!(prog.current_speed, "5.00MiB/s");
        } else {
            panic!("Failed to parse progress line");
        }
    }

    #[test]
    fn test_parse_post_processing_merger() {
        let line = "[Merger] Merging formats into \"video.mkv\"";
        assert!(matches!(
            parse_progress_line(line),
            ParsedLineEvent::PostProcessing(_)
        ));
    }
}
