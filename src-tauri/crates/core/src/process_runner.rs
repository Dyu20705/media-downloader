use std::path::Path;
use std::process::Stdio;
use tokio::io::AsyncReadExt;
use tokio::process::{Child, Command};
use tokio::sync::mpsc::Sender;

pub const OUTPUT_CHANNEL_CAPACITY: usize = 128;
const MAX_OUTPUT_LINE_BYTES: usize = 8 * 1024;

pub struct ProcessHandle {
    pub pid: Option<u32>,
    child: Option<Child>,
}

impl ProcessHandle {
    pub async fn spawn_with_streaming(
        executable: &Path,
        args: &[String],
        line_sender: Sender<String>,
    ) -> Result<Self, String> {
        let mut cmd = Command::new(executable);
        cmd.args(args);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        #[cfg(unix)]
        {
            // Put child process into a new process group so we can kill the entire tree
            cmd.process_group(0);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn child process {:?}: {}", executable, e))?;

        let pid = child.id();

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let sender = line_sender.clone();
            tokio::spawn(drain_lines(stdout, sender));
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let sender = line_sender;
            tokio::spawn(drain_lines(stderr, sender));
        }

        Ok(Self {
            pid,
            child: Some(child),
        })
    }

    pub async fn wait_for_exit(&mut self) -> Result<bool, String> {
        if let Some(mut child) = self.child.take() {
            let status = child
                .wait()
                .await
                .map_err(|e| format!("Error waiting for child process: {}", e))?;
            Ok(status.success())
        } else {
            Ok(false)
        }
    }

    pub async fn kill_tree(&mut self) {
        if let Some(pid) = self.pid {
            #[cfg(windows)]
            {
                // Kill process tree on Windows using taskkill /F /T /PID
                let pid = pid.to_string();
                let _ = std::process::Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid])
                    .output();
            }

            #[cfg(unix)]
            {
                // Kill entire process group and direct process on Unix
                unsafe {
                    let _ = libc::kill(-(pid as i32), libc::SIGKILL);
                    let _ = libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }

        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }
    }
}

/// Drain continuously, retaining at most 8 KiB per line and dropping events when
/// the bounded consumer queue is full. This keeps noisy children from blocking.
async fn drain_lines<R: tokio::io::AsyncRead + Unpin>(mut reader: R, sender: Sender<String>) {
    let mut chunk = [0_u8; 4096];
    let mut line = Vec::with_capacity(MAX_OUTPUT_LINE_BYTES);
    let mut oversized = false;
    loop {
        let count = match reader.read(&mut chunk).await {
            Ok(0) | Err(_) => break,
            Ok(count) => count,
        };
        for byte in &chunk[..count] {
            if *byte == b'\n' {
                if !oversized {
                    let text = String::from_utf8_lossy(&line)
                        .trim_end_matches('\r')
                        .to_owned();
                    let _ = sender.try_send(text);
                }
                line.clear();
                oversized = false;
            } else if line.len() < MAX_OUTPUT_LINE_BYTES {
                line.push(*byte);
            } else {
                oversized = true;
            }
        }
    }
    if !line.is_empty() && !oversized {
        let _ = sender.try_send(
            String::from_utf8_lossy(&line)
                .trim_end_matches('\r')
                .to_owned(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{drain_lines, MAX_OUTPUT_LINE_BYTES, OUTPUT_CHANNEL_CAPACITY};
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn output_drain_stays_bounded_and_does_not_block_a_noisy_writer() {
        let (mut writer, reader) = tokio::io::duplex(1024);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(OUTPUT_CHANNEL_CAPACITY);
        let drain = tokio::spawn(drain_lines(reader, sender));
        let producer = tokio::spawn(async move {
            for _ in 0..1000 {
                writer
                    .write_all(b"ordinary diagnostic line\n")
                    .await
                    .unwrap();
            }
            writer
                .write_all(&vec![b'x'; MAX_OUTPUT_LINE_BYTES + 1])
                .await
                .unwrap();
            writer.write_all(b"\nlast line\n").await.unwrap();
        });
        tokio::time::timeout(std::time::Duration::from_secs(2), producer)
            .await
            .expect("child output writer must not block on a full event channel")
            .unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(2), drain)
            .await
            .expect("reader must finish draining output")
            .unwrap();
        let retained: Vec<_> = std::iter::from_fn(|| receiver.try_recv().ok()).collect();
        assert_eq!(retained.len(), OUTPUT_CHANNEL_CAPACITY);
        assert!(retained
            .iter()
            .all(|line| line == "ordinary diagnostic line"));
    }

    #[tokio::test]
    async fn oversized_lines_are_discarded_and_following_output_is_drained() {
        let (mut writer, reader) = tokio::io::duplex(1024);
        let (sender, mut receiver) = tokio::sync::mpsc::channel(2);
        let drain = tokio::spawn(drain_lines(reader, sender));
        writer
            .write_all(&vec![b'x'; MAX_OUTPUT_LINE_BYTES + 1])
            .await
            .unwrap();
        writer.write_all(b"\nretained line\n").await.unwrap();
        drop(writer);
        drain.await.unwrap();
        assert_eq!(receiver.recv().await.as_deref(), Some("retained line"));
        assert!(receiver.recv().await.is_none());
    }
}
