# Privacy

opendownloader processes the media URL on the user's device and contacts the URL's host and the configured media tools. The application does not include an account service or analytics integration.

Download history is stored in a local SQLite database in the operating system's per-user local application-data directory. Persisted URLs are sanitized: credentials, fragments, and most query parameters are removed. Records are not encrypted by the application. If the history database cannot be opened or recovered, the app reports the error in diagnostics and uses temporary in-memory history for that session while leaving the database in place. Diagnostics are retained in a bounded in-memory buffer and redact URL paths, query strings, fragments, and common credential fields.

Managed tools are downloaded from their configured upstream sources when the user installs or repairs them. SHA-256 verification runs before installation. Source websites and media tools have their own privacy policies and network behavior.
