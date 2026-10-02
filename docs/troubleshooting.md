# Troubleshooting & Reliability Guide — openDownloader

## 1. Common Diagnostics & Solutions

### 1.1 Tool Engine Issues

| Symptom | Probable Cause | Corrective Action |
| :--- | :--- | :--- |
| **"Tool missing or damaged"** | Executable missing or checksum validation failed. | Click **Engine Tools** in header → Click **Install / Repair**. |
| **"Checksum mismatch"** | Incomplete download or tampered binary. | Click **Repair Tool** in the Tools dialog to re-download from official source. |
| **"Permission denied running executable"** | Antivirus blocked binary execution in AppData. | Add exclusion for `%LOCALAPPDATA%\opendownloader\tools\`. |

### 1.2 Download Failures

| Symptom | Probable Cause | Corrective Action |
| :--- | :--- | :--- |
| **"Unsupported URL"** | Non-HTTP/HTTPS link or unsupported site. | Verify the URL starts with `https://` and is accessible in a web browser. |
| **"Destination path write error"** | Destination folder is read-only or invalid. | Open **Settings** and choose a writable user folder like `D:\Videos` or `C:\Users\<user>\Downloads`. |
| **"Filename too long"** | Media title exceeded Windows 260 char limit. | In **Settings**, ensure **Trim Long Filenames** is enabled (defaults to 180 chars). |
| **"Post-processing failed (FFmpeg)"** | FFmpeg binary corrupted or disk out of space. | Verify disk space and run **Repair FFmpeg** in Engine Tools. |

### 1.3 Cancellation & Recovery
- **Cancelled Downloads**: The app immediately issues `taskkill /F /T /PID` on Windows or `SIGKILL` on Unix to kill all child processes. Temporary download files (`.part`, `.ytdl`) remain available for resume or can be cleared.
- **Interrupted job**: A process restart marks every queued or in-flight job Interrupted. Retry is explicit and creates a new job/output folder; partial files from the old attempt are retained and are not assumed complete.

---

## 2. Diagnostics Ring Buffer Inspection

To inspect low-level diagnostic logs:
1. Click the **Diagnostics (Terminal icon)** in the top right header.
2. Review real-time events with timestamp, log level (`INFO`, `WARN`, `ERROR`), and subsystem source (`IPC`, `TOOL_MANAGER`, `PROCESS`, `VERIFY`).
3. Click **Copy Command** to view the exact deterministic CLI invocation.
4. Click **Clear Logs** to reset the 256-entry in-memory ring buffer.
