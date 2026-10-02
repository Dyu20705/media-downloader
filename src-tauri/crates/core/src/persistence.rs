//! Durable, versioned download history. Stored URL fields are stripped of credentials,
//! fragments, and query parameters before they leave process memory.
use std::{path::Path, sync::Mutex, time::Duration};

use crate::types::{DownloadJob, DownloadStatus};
use rusqlite::{params, Connection};

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug)]
pub struct JobStore {
    connection: Mutex<Connection>,
}

impl JobStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Create database directory: {e}"))?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                    .map_err(|e| format!("Protect database directory: {e}"))?;
            }
        }
        let connection = Connection::open(path.as_ref()).map_err(db_error)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path.as_ref(), std::fs::Permissions::from_mode(0o600))
                .map_err(|e| format!("Protect database file: {e}"))?;
        }
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, String> {
        Self::initialize(Connection::open_in_memory().map_err(db_error)?)
    }

    fn initialize(connection: Connection) -> Result<Self, String> {
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(db_error)?;
        connection
            .execute_batch(
                "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
            )
            .map_err(db_error)?;
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(db_error)?;
        if version > SCHEMA_VERSION {
            return Err(format!(
                "Database schema {version} is newer than supported schema {SCHEMA_VERSION}"
            ));
        }
        if version == 0 {
            let tx = connection.unchecked_transaction().map_err(db_error)?;
            tx.execute_batch("CREATE TABLE jobs (id TEXT PRIMARY KEY NOT NULL, status TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL, can_retry INTEGER NOT NULL DEFAULT 0, payload TEXT NOT NULL); CREATE INDEX jobs_created_at ON jobs(created_at DESC); CREATE TABLE attempts (id INTEGER PRIMARY KEY, job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE, started_at TEXT NOT NULL, finished_at TEXT, outcome TEXT, error_code TEXT); PRAGMA user_version=1;").map_err(db_error)?;
            tx.commit().map_err(db_error)?;
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn save(&self, job: &DownloadJob) -> Result<(), String> {
        let mut safe = serde_json::to_value(job).map_err(|e| format!("Serialize job: {e}"))?;
        sanitize_json(&mut safe);
        let payload = serde_json::to_string(&safe).map_err(|e| format!("Serialize job: {e}"))?;
        let can_retry = safe
            .get("url")
            .and_then(|v| v.as_str())
            .is_some_and(|url| url != "[REDACTED]")
            && safe
                .get("acquisitionRequest")
                .is_some_and(|request| !request.is_null());
        let db = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        let tx = db.unchecked_transaction().map_err(db_error)?;
        let now = chrono::Utc::now().to_rfc3339();
        let exists: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM jobs WHERE id=?1)",
                [&job.id],
                |r| r.get(0),
            )
            .map_err(db_error)?;
        tx.execute("INSERT INTO jobs(id,status,created_at,updated_at,can_retry,payload) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(id) DO UPDATE SET status=excluded.status, updated_at=excluded.updated_at, can_retry=excluded.can_retry, payload=excluded.payload", params![job.id, format!("{:?}", job.status).to_uppercase(), job.created_at, now, can_retry, payload]).map_err(db_error)?;
        if !exists {
            tx.execute(
                "INSERT INTO attempts(job_id,started_at) VALUES(?1,?2)",
                params![job.id, now],
            )
            .map_err(db_error)?;
        }
        if matches!(
            job.status,
            DownloadStatus::Completed
                | DownloadStatus::Failed
                | DownloadStatus::Cancelled
                | DownloadStatus::Interrupted
        ) {
            tx.execute("UPDATE attempts SET finished_at=?2, outcome=?3 WHERE job_id=?1 AND finished_at IS NULL", params![job.id, now, format!("{:?}", job.status).to_uppercase()]).map_err(db_error)?;
        }
        tx.commit().map_err(db_error)?;
        Ok(())
    }

    pub fn list(&self, limit: usize) -> Result<Vec<DownloadJob>, String> {
        let db = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        let mut statement = db
            .prepare("SELECT payload FROM jobs ORDER BY created_at DESC LIMIT ?1")
            .map_err(db_error)?;
        let rows = statement
            .query_map([limit.min(500) as i64], |row| row.get::<_, String>(0))
            .map_err(db_error)?;
        rows.map(|row| {
            let payload = row.map_err(db_error)?;
            serde_json::from_str(&payload).map_err(|e| format!("Stored job record is invalid: {e}"))
        })
        .collect()
    }

    /// Any process-owned state is stale after restart. Keep the record and require an explicit user retry.
    pub fn recover_interrupted(&self) -> Result<usize, String> {
        let db = self
            .connection
            .lock()
            .map_err(|_| "Database lock poisoned".to_string())?;
        let tx = db.unchecked_transaction().map_err(db_error)?;
        let rows = {
            let mut stmt = tx
                .prepare("SELECT id,status,payload FROM jobs ORDER BY created_at DESC")
                .map_err(db_error)?;
            let rows = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })
                .map_err(db_error)?;
            rows.collect::<Result<Vec<_>, _>>().map_err(db_error)?
        };
        let mut changed = 0;
        for (id, status, payload) in rows {
            let mut value: serde_json::Value = serde_json::from_str(&payload)
                .map_err(|e| format!("Stored job record {id} is malformed: {e}"))?;
            match status.as_str() {
                "QUEUED" | "PREPARING" | "DOWNLOADING" | "POSTPROCESSING" | "VERIFYING"
                | "CANCELLING" => {
                    value["status"] = serde_json::Value::String("INTERRUPTED".into());
                    value["errorMessage"] = serde_json::Value::String("This download was interrupted when the application closed. Review it before retrying.".into());
                    let serialized = serde_json::to_string(&value)
                        .map_err(|e| format!("Serialize recovered job: {e}"))?;
                    tx.execute(
                        "UPDATE jobs SET status='INTERRUPTED',updated_at=?2,payload=?3 WHERE id=?1",
                        params![id, chrono::Utc::now().to_rfc3339(), serialized],
                    )
                    .map_err(db_error)?;
                    tx.execute("UPDATE attempts SET finished_at=?2,outcome='INTERRUPTED' WHERE job_id=?1 AND finished_at IS NULL", params![id, chrono::Utc::now().to_rfc3339()]).map_err(db_error)?;
                    changed += 1;
                }
                "IDLE" | "ANALYZING" | "READY" | "COMPLETED" | "FAILED" | "CANCELLED"
                | "INTERRUPTED" => {}
                invalid => return Err(format!("Stored job {id} has invalid state {invalid}")),
            }
        }
        tx.commit().map_err(db_error)?;
        Ok(changed)
    }
}

fn sanitize_json(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text)
            if text.starts_with("http://") || text.starts_with("https://") =>
        {
            *text = safe_url(text)
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(sanitize_json),
        serde_json::Value::Object(fields) => fields.values_mut().for_each(sanitize_json),
        _ => {}
    }
}
fn safe_url(input: &str) -> String {
    match url::Url::parse(input) {
        Ok(mut url) if matches!(url.scheme(), "http" | "https") => {
            let host = url.host_str().unwrap_or_default().to_ascii_lowercase();
            let allowed: &[&str] =
                if host == "youtube.com" || host.ends_with(".youtube.com") || host == "youtu.be" {
                    &["v", "list", "index", "start", "t"]
                } else {
                    &[]
                };
            let retained: Vec<(String, String)> = url
                .query_pairs()
                .filter(|(key, _)| allowed.contains(&key.as_ref()))
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect();
            let _ = url.set_username("");
            let _ = url.set_password(None);
            url.set_query(None);
            url.set_fragment(None);
            if !retained.is_empty() {
                url.query_pairs_mut().extend_pairs(retained);
            }
            url.to_string()
        }
        _ => "[REDACTED]".into(),
    }
}
fn db_error(error: rusqlite::Error) -> String {
    format!("Download history database error: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_database_has_versioned_schema_and_recovery_is_stable() {
        let store = JobStore::open_in_memory().unwrap();
        assert_eq!(store.recover_interrupted().unwrap(), 0);
        assert!(store.list(20).unwrap().is_empty());
        let db = store.connection.lock().unwrap();
        let version: i64 = db
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        let table: String = db
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='attempts'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(table, "attempts");
    }

    #[cfg(unix)]
    #[test]
    fn database_files_are_private_to_the_current_user() {
        use std::os::unix::fs::PermissionsExt;
        let directory = tempfile::tempdir().unwrap();
        let database = directory.path().join("private").join("jobs.sqlite3");
        let _store = JobStore::open(&database).unwrap();
        assert_eq!(
            std::fs::metadata(database.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        assert_eq!(
            std::fs::metadata(database).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

    #[test]
    fn url_persistence_strips_secrets_and_query_tokens() {
        let clean = safe_url("https://user:pass@example.com/watch?v=abc&token=secret#fragment");
        assert_eq!(clean, "https://example.com/watch");
        assert!(!clean.contains("secret"));
        assert_eq!(
            safe_url("https://www.youtube.com/watch?v=abc&list=playlist&token=secret"),
            "https://www.youtube.com/watch?v=abc&list=playlist"
        );
    }
    #[test]
    fn recovery_marks_running_jobs_interrupted_once_and_rejects_invalid_state() {
        let store = JobStore::open_in_memory().unwrap();
        {
            let db = store.connection.lock().unwrap();
            db.execute(
                "INSERT INTO jobs(id,status,created_at,updated_at,payload) VALUES(?1,?2,?3,?3,?4)",
                rusqlite::params!["j1", "DOWNLOADING", "now", r#"{"status":"DOWNLOADING"}"#],
            )
            .unwrap();
            db.execute(
                "INSERT INTO attempts(job_id,started_at) VALUES('j1','now')",
                [],
            )
            .unwrap();
        }
        assert_eq!(store.recover_interrupted().unwrap(), 1);
        assert_eq!(store.recover_interrupted().unwrap(), 0);
        let db = store.connection.lock().unwrap();
        let status: String = db
            .query_row("SELECT status FROM jobs WHERE id='j1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(status, "INTERRUPTED");
        db.execute(
            "INSERT INTO jobs(id,status,created_at,updated_at,payload) VALUES(?1,?2,?3,?3,?4)",
            rusqlite::params!["j2", "BOGUS", "now", "{}"],
        )
        .unwrap();
        drop(db);
        assert!(store
            .recover_interrupted()
            .unwrap_err()
            .contains("invalid state"));
    }

    #[test]
    fn newer_schema_is_rejected_without_downgrade() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .unwrap();
        assert!(JobStore::initialize(connection)
            .unwrap_err()
            .contains("newer than supported"));
    }
}
