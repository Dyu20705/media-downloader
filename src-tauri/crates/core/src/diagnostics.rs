use crate::types::DiagnosticLog;
use chrono::Local;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

const MAX_LOG_LINES: usize = 256;
const MAX_TOTAL_BYTES: usize = 64 * 1024; // <= 64 KiB retained text

#[derive(Debug, Clone)]
pub struct DiagnosticsBuffer {
    logs: Arc<RwLock<VecDeque<DiagnosticLog>>>,
    retained_bytes: Arc<RwLock<usize>>,
}

impl Default for DiagnosticsBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsBuffer {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(RwLock::new(VecDeque::with_capacity(MAX_LOG_LINES))),
            retained_bytes: Arc::new(RwLock::new(0)),
        }
    }

    pub fn log(&self, level: &str, source: &str, raw_message: &str) {
        let sanitized = sanitize_diagnostic_text(raw_message);
        let now_str = Local::now().format("%H:%M:%S").to_string();
        let msg_len = sanitized.len() + level.len() + source.len() + 32;

        let log_entry = DiagnosticLog {
            id: format!("{}-{}", Local::now().timestamp_micros(), source),
            timestamp: now_str,
            level: level.to_string(),
            source: source.to_string(),
            message: sanitized,
        };

        if let (Ok(mut lock), Ok(mut bytes_lock)) = (self.logs.write(), self.retained_bytes.write())
        {
            // Trim by line count
            while lock.len() >= MAX_LOG_LINES {
                if let Some(removed) = lock.pop_front() {
                    let removed_len =
                        removed.message.len() + removed.level.len() + removed.source.len() + 32;
                    *bytes_lock = bytes_lock.saturating_sub(removed_len);
                }
            }

            // Trim by total retained memory bytes (<= 64 KiB)
            while *bytes_lock + msg_len > MAX_TOTAL_BYTES && !lock.is_empty() {
                if let Some(removed) = lock.pop_front() {
                    let removed_len =
                        removed.message.len() + removed.level.len() + removed.source.len() + 32;
                    *bytes_lock = bytes_lock.saturating_sub(removed_len);
                }
            }

            *bytes_lock += msg_len;
            lock.push_back(log_entry);
        }
    }

    pub fn get_logs(&self) -> Vec<DiagnosticLog> {
        self.logs
            .read()
            .map(|lock| lock.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let (Ok(mut lock), Ok(mut bytes_lock)) = (self.logs.write(), self.retained_bytes.write())
        {
            lock.clear();
            *bytes_lock = 0;
        }
    }
}

pub fn sanitize_diagnostic_text(text: &str) -> String {
    // Redact common auth tokens, passwords, cookies, signed URL tokens, private headers
    let mut result = text.to_string();

    let sensitive_patterns = [
        ("(?i)bearer [a-zA-Z0-9_\\-\\.]+", "Bearer [REDACTED]"),
        ("(?i)authorization: [^\\s]+", "Authorization: [REDACTED]"),
        ("(?i)cookie: [^\\s]+", "Cookie: [REDACTED]"),
        ("(?i)api_key=[^&\\s]+", "api_key=[REDACTED]"),
        ("(?i)apikey=[^&\\s]+", "apikey=[REDACTED]"),
        ("(?i)password=[^&\\s]+", "password=[REDACTED]"),
        ("(?i)secret=[^&\\s]+", "secret=[REDACTED]"),
        ("(?i)sig=[a-zA-Z0-9_\\-\\.]+", "sig=[REDACTED]"),
        ("(?i)signature=[a-zA-Z0-9_\\-\\.]+", "signature=[REDACTED]"),
        ("(?i)token=[a-zA-Z0-9_\\-\\.]+", "token=[REDACTED]"),
    ];

    for (pattern, replacement) in sensitive_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            result = re.replace_all(&result, replacement).to_string();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitization() {
        let dirty = "Executing request with Bearer secret123456 and api_key=supersecret and token=abc1234xyz";
        let cleaned = sanitize_diagnostic_text(dirty);
        assert!(!cleaned.contains("secret123456"));
        assert!(!cleaned.contains("supersecret"));
        assert!(!cleaned.contains("abc1234xyz"));
        assert!(cleaned.contains("[REDACTED]"));
    }

    #[test]
    fn test_bounded_ring_buffer() {
        let buffer = DiagnosticsBuffer::new();
        for i in 0..300 {
            buffer.log("INFO", "TEST", &format!("Log message {}", i));
        }

        let logs = buffer.get_logs();
        assert_eq!(logs.len(), MAX_LOG_LINES);
        // Oldest logs (0..43) dropped, latest remain
        assert!(logs.last().unwrap().message.contains("Log message 299"));
    }
}
