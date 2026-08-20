use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc::UnboundedSender;

pub struct ProcessHandle {
    pub pid: Option<u32>,
    child: Option<Child>,
}

impl ProcessHandle {
    pub async fn spawn_with_streaming(
        executable: &Path,
        args: &[String],
        line_sender: UnboundedSender<String>,
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
            tokio::spawn(async move {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender.send(line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let sender = line_sender;
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    let _ = sender.send(line);
                }
            });
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
                let _ = std::process::Command::new("taskkill")
                    .args(&["/F", "/T", "/PID", &pid.to_string()])
                    .output();
            }

            #[cfg(unix)]
            {
                // Kill entire process group on Unix
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
            }
        }

        if let Some(mut child) = self.child.take() {
            let _ = child.kill().await;
        }
    }
}
