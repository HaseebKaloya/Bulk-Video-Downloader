use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use crate::domain::{
    AppSettings, Batch, BatchStatus, CompletionPolicy, DownloadProfile,
    DownloadStatus, DownloadTask,
};
use crate::errors::AppError;

pub struct Database {
    conn: Arc<Mutex<Connection>>,
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AppError> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| AppError::FilesystemError {
                code: "FS_CREATE_DIR_FAILED".into(),
                message: format!("Failed to create database directory: {}", e),
                action: "Ensure adequate permissions on the application data path.".into(),
            })?;
        }

        let conn = Connection::open(&db_path).map_err(|e| AppError::DatabaseError {
            code: "DB_OPEN_FAILED".into(),
            message: format!("Failed to open SQLite database at {:?}: {}", db_path, e),
            action: "Check disk permissions and ensure disk is not full.".into(),
        })?;

        // Configure WAL mode and foreign keys
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_PRAGMA_FAILED".into(),
            message: format!("Failed to configure SQLite pragmas: {}", e),
            action: "Check database integrity.".into(),
        })?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        db.run_migrations()?;
        db.seed_defaults_if_empty()?;

        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, AppError> {
        let conn = Connection::open_in_memory().map_err(|e| AppError::DatabaseError {
            code: "DB_OPEN_FAILED".into(),
            message: format!("Failed to open in-memory SQLite: {}", e),
            action: "Check memory availability.".into(),
        })?;

        conn.execute_batch(
            "PRAGMA foreign_keys = ON;",
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_PRAGMA_FAILED".into(),
            message: format!("Failed to configure in-memory SQLite pragmas: {}", e),
            action: "Check system memory.".into(),
        })?;

        let db = Database {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        };

        db.run_migrations()?;
        db.seed_defaults_if_empty()?;

        Ok(db)
    }

    pub fn check_integrity(&self) -> Result<bool, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("PRAGMA integrity_check;")
            .map_err(|e| AppError::DatabaseError {
                code: "DB_INTEGRITY_CHECK_FAILED".into(),
                message: format!("Failed to prepare integrity check: {}", e),
                action: "Restore database from backup or restart application.".into(),
            })?;

        let result: String = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| AppError::DatabaseError {
                code: "DB_INTEGRITY_CHECK_FAILED".into(),
                message: format!("Failed to execute integrity check: {}", e),
                action: "Restore database from backup.".into(),
            })?;

        Ok(result == "ok")
    }

    fn run_migrations(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL
            );",
            [],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "MIGRATION_FAILED".into(),
            message: format!("Failed to initialize schema_version table: {}", e),
            action: "Check database permissions.".into(),
        })?;

        let current_version: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version;",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        if current_version < 1 {
            conn.execute_batch(
                "BEGIN TRANSACTION;

                CREATE TABLE IF NOT EXISTS batches (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    status TEXT NOT NULL,
                    batch_size INTEGER NOT NULL,
                    concurrency INTEGER NOT NULL,
                    completion_policy TEXT NOT NULL,
                    failure_policy TEXT NOT NULL,
                    profile_id TEXT,
                    destination_directory TEXT NOT NULL,
                    naming_template TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    started_at TEXT,
                    completed_at TEXT,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS downloads (
                    id TEXT PRIMARY KEY,
                    batch_id TEXT NOT NULL,
                    source_url TEXT NOT NULL,
                    provider_id TEXT,
                    source_id TEXT,
                    title TEXT,
                    thumbnail_url TEXT,
                    status TEXT NOT NULL,
                    priority INTEGER NOT NULL DEFAULT 0,
                    queue_position INTEGER NOT NULL,
                    selected_format_id TEXT,
                    selected_quality TEXT,
                    selected_extension TEXT,
                    output_directory TEXT NOT NULL,
                    output_filename TEXT,
                    output_path TEXT,
                    partial_path TEXT,
                    bytes_downloaded INTEGER NOT NULL DEFAULT 0,
                    total_bytes INTEGER,
                    progress REAL NOT NULL DEFAULT 0,
                    speed_bytes_per_second REAL,
                    eta_seconds INTEGER,
                    retry_count INTEGER NOT NULL DEFAULT 0,
                    max_retries INTEGER NOT NULL DEFAULT 3,
                    next_retry_at TEXT,
                    last_error_code TEXT,
                    last_error_message TEXT,
                    created_at TEXT NOT NULL,
                    started_at TEXT,
                    completed_at TEXT,
                    updated_at TEXT NOT NULL,
                    FOREIGN KEY (batch_id) REFERENCES batches(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS download_events (
                    id TEXT PRIMARY KEY,
                    download_id TEXT NOT NULL,
                    event_type TEXT NOT NULL,
                    old_status TEXT,
                    new_status TEXT,
                    message TEXT,
                    metadata_json TEXT,
                    created_at TEXT NOT NULL,
                    FOREIGN KEY (download_id) REFERENCES downloads(id) ON DELETE CASCADE
                );

                CREATE TABLE IF NOT EXISTS profiles (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    quality_policy TEXT NOT NULL,
                    format_policy TEXT NOT NULL,
                    destination_directory TEXT NOT NULL,
                    naming_template TEXT NOT NULL,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value_json TEXT NOT NULL,
                    updated_at TEXT NOT NULL
                );

                CREATE INDEX IF NOT EXISTS idx_downloads_batch_status ON downloads(batch_id, status);
                CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
                CREATE INDEX IF NOT EXISTS idx_downloads_source_url ON downloads(source_url);
                CREATE INDEX IF NOT EXISTS idx_downloads_retry_time ON downloads(next_retry_at);
                CREATE INDEX IF NOT EXISTS idx_download_events_download ON download_events(download_id, created_at);

                INSERT INTO schema_version (version, applied_at) VALUES (1, datetime('now'));

                COMMIT;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "MIGRATION_V1_FAILED".into(),
                message: format!("Failed to execute migration v1: {}", e),
                action: "Check disk space and SQLite file permissions.".into(),
            })?;
        }

        Ok(())
    }

    fn seed_defaults_if_empty(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();

        let profile_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM profiles;", [], |r| r.get(0))
            .unwrap_or(0);

        if profile_count == 0 {
            let now = Utc::now().to_rfc3339();
            let default_dir = dirs::download_dir()
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| ".".into()))
                .to_string_lossy()
                .to_string();

            let profiles = vec![
                ("highest-available", "Highest Available (Best)", "highest", "mp4", "{title}_{quality}.{format}"),
                ("balanced-1080p", "Balanced (1080p Full HD)", "1080p", "mp4", "{title}_1080p.{format}"),
                ("standard-720p", "Standard (720p HD)", "720p", "mp4", "{title}_720p.{format}"),
                ("small-file", "Small File (480p)", "480p", "mp4", "{title}_small.{format}"),
                ("audio-only", "Audio Only (MP3)", "audio", "mp3", "{title}_audio.{format}"),
            ];

            for (id, name, q, f, t) in profiles {
                let _ = conn.execute(
                    "INSERT INTO profiles (id, name, quality_policy, format_policy, destination_directory, naming_template, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8);",
                    params![id, name, q, f, default_dir, t, now, now],
                );
            }
        }

        // Seed settings if missing
        let settings_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM settings WHERE key = 'app_settings';", [], |r| r.get(0))
            .unwrap_or(0);

        if settings_count == 0 {
            let default_settings = AppSettings::default();
            let json = serde_json::to_string(&default_settings).unwrap_or_default();
            let now = Utc::now().to_rfc3339();
            let _ = conn.execute(
                "INSERT INTO settings (key, value_json, updated_at) VALUES ('app_settings', ?1, ?2);",
                params![json, now],
            );
        }

        Ok(())
    }

    // --- BATCH REPOSITORY ---

    pub fn insert_batch(&self, batch: &Batch) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO batches (
                id, name, status, batch_size, concurrency, completion_policy,
                failure_policy, profile_id, destination_directory, naming_template,
                created_at, started_at, completed_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14);",
            params![
                batch.id,
                batch.name,
                batch.status.as_str(),
                batch.batch_size,
                batch.concurrency,
                batch.completion_policy.as_str(),
                batch.failure_policy,
                batch.profile_id,
                batch.destination_directory,
                batch.naming_template,
                batch.created_at,
                batch.started_at,
                batch.completed_at,
                batch.updated_at
            ],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_INSERT_BATCH_FAILED".into(),
            message: format!("Failed to insert batch: {}", e),
            action: "Check batch identifier uniqueness.".into(),
        })?;
        Ok(())
    }

    pub fn get_batch(&self, id: &str) -> Result<Option<Batch>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, status, batch_size, concurrency, completion_policy,
                        failure_policy, profile_id, destination_directory, naming_template,
                        created_at, started_at, completed_at, updated_at
                 FROM batches WHERE id = ?1;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to prepare batch query: {}", e),
                action: "Retry operation.".into(),
            })?;

        let batch = stmt
            .query_row(params![id], |row| {
                let status_str: String = row.get(2)?;
                let policy_str: String = row.get(5)?;
                Ok(Batch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: BatchStatus::from_str(&status_str),
                    batch_size: row.get(3)?,
                    concurrency: row.get(4)?,
                    completion_policy: CompletionPolicy::from_str(&policy_str),
                    failure_policy: row.get(6)?,
                    profile_id: row.get(7)?,
                    destination_directory: row.get(8)?,
                    naming_template: row.get(9)?,
                    created_at: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            })
            .optional()
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to get batch: {}", e),
                action: "Verify batch ID.".into(),
            })?;

        Ok(batch)
    }

    pub fn list_batches(&self) -> Result<Vec<Batch>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, status, batch_size, concurrency, completion_policy,
                        failure_policy, profile_id, destination_directory, naming_template,
                        created_at, started_at, completed_at, updated_at
                 FROM batches ORDER BY created_at DESC;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to prepare list batches query: {}", e),
                action: "Retry operation.".into(),
            })?;

        let rows = stmt
            .query_map([], |row| {
                let status_str: String = row.get(2)?;
                let policy_str: String = row.get(5)?;
                Ok(Batch {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    status: BatchStatus::from_str(&status_str),
                    batch_size: row.get(3)?,
                    concurrency: row.get(4)?,
                    completion_policy: CompletionPolicy::from_str(&policy_str),
                    failure_policy: row.get(6)?,
                    profile_id: row.get(7)?,
                    destination_directory: row.get(8)?,
                    naming_template: row.get(9)?,
                    created_at: row.get(10)?,
                    started_at: row.get(11)?,
                    completed_at: row.get(12)?,
                    updated_at: row.get(13)?,
                })
            })
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to query batches: {}", e),
                action: "Retry operation.".into(),
            })?;

        let mut batches = Vec::new();
        for r in rows {
            if let Ok(b) = r {
                batches.push(b);
            }
        }
        Ok(batches)
    }

    pub fn update_batch_status(&self, id: &str, status: BatchStatus) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE batches SET status = ?1, updated_at = ?2 WHERE id = ?3;",
            params![status.as_str(), now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_UPDATE_FAILED".into(),
            message: format!("Failed to update batch status: {}", e),
            action: "Check batch ID.".into(),
        })?;
        Ok(())
    }

    pub fn update_batch_config(
        &self,
        id: &str,
        batch_size: u32,
        concurrency: u32,
        completion_policy: CompletionPolicy,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE batches SET batch_size = ?1, concurrency = ?2, completion_policy = ?3, updated_at = ?4 WHERE id = ?5;",
            params![batch_size, concurrency, completion_policy.as_str(), now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_UPDATE_FAILED".into(),
            message: format!("Failed to update batch config: {}", e),
            action: "Ensure valid parameters.".into(),
        })?;
        Ok(())
    }

    pub fn delete_batch(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute("DELETE FROM downloads WHERE batch_id = ?1;", params![id]);
        conn.execute("DELETE FROM batches WHERE id = ?1;", params![id])
            .map_err(|e| AppError::DatabaseError {
                code: "DB_DELETE_FAILED".into(),
                message: format!("Failed to delete batch: {}", e),
                action: "Verify batch ID.".into(),
            })?;
        Ok(())
    }

    pub fn delete_batches(&self, ids: &[String]) -> Result<(), AppError> {
        if ids.is_empty() {
            return Ok(());
        }
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| AppError::DatabaseError {
            code: "DB_TX_FAILED".into(),
            message: format!("Failed to begin transaction: {}", e),
            action: "Retry operation.".into(),
        })?;
        for id in ids {
            let _ = tx.execute("DELETE FROM downloads WHERE batch_id = ?1;", params![id]);
            let _ = tx.execute("DELETE FROM batches WHERE id = ?1;", params![id]);
        }
        tx.commit().map_err(|e| AppError::DatabaseError {
            code: "DB_COMMIT_FAILED".into(),
            message: format!("Failed to commit batch deletions: {}", e),
            action: "Retry operation.".into(),
        })?;
        Ok(())
    }

    pub fn clear_all_batches(&self) -> Result<(), AppError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| AppError::DatabaseError {
            code: "DB_TX_FAILED".into(),
            message: format!("Failed to begin transaction: {}", e),
            action: "Retry operation.".into(),
        })?;
        let _ = tx.execute("DELETE FROM downloads;", []);
        let _ = tx.execute("DELETE FROM batches;", []);
        tx.commit().map_err(|e| AppError::DatabaseError {
            code: "DB_CLEAR_FAILED".into(),
            message: format!("Failed to clear all batches: {}", e),
            action: "Retry operation.".into(),
        })?;
        Ok(())
    }

    // --- DOWNLOAD TASK REPOSITORY ---

    pub fn insert_downloads(&self, tasks: &[DownloadTask]) -> Result<(), AppError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction().map_err(|e| AppError::DatabaseError {
            code: "DB_TX_FAILED".into(),
            message: format!("Failed to begin transaction: {}", e),
            action: "Check database integrity.".into(),
        })?;

        for task in tasks {
            tx.execute(
                "INSERT INTO downloads (
                    id, batch_id, source_url, provider_id, source_id, title, thumbnail_url,
                    status, priority, queue_position, selected_format_id, selected_quality,
                    selected_extension, output_directory, output_filename, output_path,
                    partial_path, bytes_downloaded, total_bytes, progress, speed_bytes_per_second,
                    eta_seconds, retry_count, max_retries, next_retry_at, last_error_code,
                    last_error_message, created_at, started_at, completed_at, updated_at
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28,
                    ?29, ?30, ?31
                );",
                params![
                    task.id,
                    task.batch_id,
                    task.source_url,
                    task.provider_id,
                    task.source_id,
                    task.title,
                    task.thumbnail_url,
                    task.status.as_str(),
                    task.priority,
                    task.queue_position,
                    task.selected_format_id,
                    task.selected_quality,
                    task.selected_extension,
                    task.output_directory,
                    task.output_filename,
                    task.output_path,
                    task.partial_path,
                    task.bytes_downloaded,
                    task.total_bytes,
                    task.progress,
                    task.speed_bytes_per_second,
                    task.eta_seconds,
                    task.retry_count,
                    task.max_retries,
                    task.next_retry_at,
                    task.last_error_code,
                    task.last_error_message,
                    task.created_at,
                    task.started_at,
                    task.completed_at,
                    task.updated_at
                ],
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_INSERT_DOWNLOAD_FAILED".into(),
                message: format!("Failed to insert download task {}: {}", task.id, e),
                action: "Check task constraints.".into(),
            })?;
        }

        tx.commit().map_err(|e| AppError::DatabaseError {
            code: "DB_COMMIT_FAILED".into(),
            message: format!("Failed to commit batch download insert: {}", e),
            action: "Retry operation.".into(),
        })?;

        Ok(())
    }

    pub fn get_download(&self, id: &str) -> Result<Option<DownloadTask>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, batch_id, source_url, provider_id, source_id, title, thumbnail_url,
                        status, priority, queue_position, selected_format_id, selected_quality,
                        selected_extension, output_directory, output_filename, output_path,
                        partial_path, bytes_downloaded, total_bytes, progress, speed_bytes_per_second,
                        eta_seconds, retry_count, max_retries, next_retry_at, last_error_code,
                        last_error_message, created_at, started_at, completed_at, updated_at
                 FROM downloads WHERE id = ?1;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to prepare download task query: {}", e),
                action: "Retry operation.".into(),
            })?;

        let task = stmt
            .query_row(params![id], Self::map_download_row)
            .optional()
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to get download: {}", e),
                action: "Verify task ID.".into(),
            })?;

        Ok(task)
    }

    pub fn list_downloads(
        &self,
        batch_id: Option<&str>,
        status: Option<&str>,
        search: Option<&str>,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<DownloadTask>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut sql = "SELECT id, batch_id, source_url, provider_id, source_id, title, thumbnail_url,
                              status, priority, queue_position, selected_format_id, selected_quality,
                              selected_extension, output_directory, output_filename, output_path,
                              partial_path, bytes_downloaded, total_bytes, progress, speed_bytes_per_second,
                              eta_seconds, retry_count, max_retries, next_retry_at, last_error_code,
                              last_error_message, created_at, started_at, completed_at, updated_at
                       FROM downloads WHERE 1=1".to_string();

        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(b_id) = batch_id {
            sql.push_str(" AND batch_id = ?");
            params_vec.push(Box::new(b_id.to_string()));
        }

        if let Some(st) = status {
            sql.push_str(" AND status = ?");
            params_vec.push(Box::new(st.to_string()));
        }

        if let Some(q) = search {
            if !q.trim().is_empty() {
                sql.push_str(" AND (title LIKE ? OR source_url LIKE ? OR output_filename LIKE ?)");
                let pattern = format!("%{}%", q.trim());
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern.clone()));
                params_vec.push(Box::new(pattern));
            }
        }

        match status {
            Some("COMPLETED") => {
                sql.push_str(" ORDER BY completed_at DESC, updated_at DESC, created_at DESC");
            }
            Some("QUEUED") | Some("RETRY_WAIT") => {
                sql.push_str(" ORDER BY priority DESC, queue_position ASC");
            }
            _ => {
                sql.push_str(" ORDER BY \
                    CASE \
                        WHEN status IN ('DOWNLOADING', 'PREPARING', 'FINALIZING') THEN 0 \
                        WHEN status IN ('QUEUED', 'RETRY_WAIT') THEN 1 \
                        WHEN status = 'PAUSED' THEN 2 \
                        WHEN status = 'COMPLETED' THEN 3 \
                        ELSE 4 \
                    END ASC, \
                    CASE \
                        WHEN status IN ('QUEUED', 'RETRY_WAIT') THEN priority \
                        ELSE 0 \
                    END DESC, \
                    CASE \
                        WHEN status IN ('QUEUED', 'RETRY_WAIT') THEN queue_position \
                        ELSE 0 \
                    END ASC, \
                    CASE \
                        WHEN status = 'COMPLETED' THEN completed_at \
                        ELSE updated_at \
                    END DESC, \
                    created_at DESC");
            }
        }

        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
            if let Some(off) = offset {
                sql.push_str(&format!(" OFFSET {}", off));
            }
        }

        let mut stmt = conn.prepare(&sql).map_err(|e| AppError::DatabaseError {
            code: "DB_QUERY_FAILED".into(),
            message: format!("Failed to prepare downloads query: {}", e),
            action: "Check filter syntax.".into(),
        })?;

        let params_slice: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| &**p).collect();
        let rows = stmt
            .query_map(&*params_slice, Self::map_download_row)
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to query downloads: {}", e),
                action: "Retry operation.".into(),
            })?;

        let mut tasks = Vec::new();
        for r in rows {
            if let Ok(t) = r {
                tasks.push(t);
            }
        }
        Ok(tasks)
    }

    pub fn update_download_status(
        &self,
        id: &str,
        status: DownloadStatus,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        let completed_at = if status == DownloadStatus::Completed {
            Some(now.clone())
        } else {
            None
        };

        conn.execute(
            "UPDATE downloads SET
                status = ?1,
                last_error_code = COALESCE(?2, last_error_code),
                last_error_message = COALESCE(?3, last_error_message),
                completed_at = COALESCE(?4, completed_at),
                updated_at = ?5
             WHERE id = ?6;",
            params![status.as_str(), error_code, error_message, completed_at, now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_UPDATE_FAILED".into(),
            message: format!("Failed to update task status for {}: {}", id, e),
            action: "Check task ID.".into(),
        })?;
        Ok(())
    }

    pub fn update_download_progress(
        &self,
        id: &str,
        bytes_downloaded: u64,
        total_bytes: Option<u64>,
        progress: f64,
        speed: Option<f64>,
        eta: Option<u64>,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE downloads SET
                bytes_downloaded = ?1,
                total_bytes = COALESCE(?2, total_bytes),
                progress = ?3,
                speed_bytes_per_second = ?4,
                eta_seconds = ?5,
                updated_at = ?6
             WHERE id = ?7;",
            params![bytes_downloaded, total_bytes, progress, speed, eta, now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_UPDATE_PROGRESS_FAILED".into(),
            message: format!("Failed to update task progress: {}", e),
            action: "Retry operation.".into(),
        })?;
        Ok(())
    }

    pub fn update_download_final_file(
        &self,
        id: &str,
        output_filename: &str,
        output_path: &str,
        total_bytes: u64,
    ) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE downloads SET
                output_filename = ?1,
                output_path = ?2,
                bytes_downloaded = ?3,
                total_bytes = ?3,
                progress = 1.0,
                status = 'COMPLETED',
                completed_at = ?4,
                updated_at = ?4
             WHERE id = ?5;",
            params![output_filename, output_path, total_bytes, now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_FINAL_UPDATE_FAILED".into(),
            message: format!("Failed to record final download file: {}", e),
            action: "Check database integrity.".into(),
        })?;
        Ok(())
    }

    pub fn increment_retry_count(
        &self,
        id: &str,
        next_retry_at: Option<&str>,
        error_code: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<u32, AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE downloads SET
                retry_count = retry_count + 1,
                status = 'RETRY_WAIT',
                next_retry_at = ?1,
                last_error_code = ?2,
                last_error_message = ?3,
                updated_at = ?4
             WHERE id = ?5;",
            params![next_retry_at, error_code, error_message, now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_RETRY_UPDATE_FAILED".into(),
            message: format!("Failed to update retry state: {}", e),
            action: "Check task ID.".into(),
        })?;

        let count: u32 = conn
            .query_row("SELECT retry_count FROM downloads WHERE id = ?1;", params![id], |r| r.get(0))
            .unwrap_or(0);
        Ok(count)
    }

    pub fn delete_download(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM downloads WHERE id = ?1;", params![id])
            .map_err(|e| AppError::DatabaseError {
                code: "DB_DELETE_FAILED".into(),
                message: format!("Failed to delete download: {}", e),
                action: "Check task ID.".into(),
            })?;
        Ok(())
    }

    pub fn change_priority(&self, id: &str, priority: i32) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE downloads SET priority = ?1, updated_at = ?2 WHERE id = ?3;",
            params![priority, now, id],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_UPDATE_FAILED".into(),
            message: format!("Failed to change task priority: {}", e),
            action: "Check task ID.".into(),
        })?;
        Ok(())
    }

    pub fn check_existing_url(&self, url: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM downloads WHERE source_url = ?1 AND status NOT IN ('FAILED', 'CANCELLED');",
                params![url],
                |r| r.get(0),
            )
            .unwrap_or(0);
        Ok(count > 0)
    }

    // --- SETTINGS REPOSITORY ---

    pub fn get_settings(&self) -> Result<AppSettings, AppError> {
        let conn = self.conn.lock().unwrap();
        let json: Option<String> = conn
            .query_row(
                "SELECT value_json FROM settings WHERE key = 'app_settings';",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| AppError::DatabaseError {
                code: "DB_SETTINGS_FAILED".into(),
                message: format!("Failed to load settings: {}", e),
                action: "Check database integrity.".into(),
            })?;

        if let Some(j) = json {
            serde_json::from_str(&j).map_err(|e| AppError::ConfigurationError {
                code: "SETTINGS_PARSE_FAILED".into(),
                message: format!("Corrupted settings JSON: {}", e),
                action: "Reset settings to default.".into(),
            })
        } else {
            Ok(AppSettings::default())
        }
    }

    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let json = serde_json::to_string(settings).map_err(|e| AppError::ConfigurationError {
            code: "SETTINGS_SERIALIZE_FAILED".into(),
            message: format!("Failed to serialize settings: {}", e),
            action: "Check settings configuration values.".into(),
        })?;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO settings (key, value_json, updated_at)
             VALUES ('app_settings', ?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at;",
            params![json, now],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_SETTINGS_SAVE_FAILED".into(),
            message: format!("Failed to save settings: {}", e),
            action: "Check disk permissions.".into(),
        })?;

        Ok(())
    }

    // --- PROFILES REPOSITORY ---

    pub fn list_profiles(&self) -> Result<Vec<DownloadProfile>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, quality_policy, format_policy, destination_directory, naming_template, created_at, updated_at
                 FROM profiles ORDER BY name ASC;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to prepare profiles query: {}", e),
                action: "Retry operation.".into(),
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok(DownloadProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quality_policy: row.get(2)?,
                    format_policy: row.get(3)?,
                    destination_directory: row.get(4)?,
                    naming_template: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to query profiles: {}", e),
                action: "Retry operation.".into(),
            })?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(p) = r {
                list.push(p);
            }
        }
        Ok(list)
    }

    pub fn save_profile(&self, profile: &DownloadProfile) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO profiles (id, name, quality_policy, format_policy, destination_directory, naming_template, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                quality_policy = excluded.quality_policy,
                format_policy = excluded.format_policy,
                destination_directory = excluded.destination_directory,
                naming_template = excluded.naming_template,
                updated_at = excluded.updated_at;",
            params![
                profile.id,
                profile.name,
                profile.quality_policy,
                profile.format_policy,
                profile.destination_directory,
                profile.naming_template,
                profile.created_at,
                now
            ],
        )
        .map_err(|e| AppError::DatabaseError {
            code: "DB_PROFILE_SAVE_FAILED".into(),
            message: format!("Failed to save profile: {}", e),
            action: "Check profile ID uniqueness.".into(),
        })?;
        Ok(())
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM profiles WHERE id = ?1;", params![id])
            .map_err(|e| AppError::DatabaseError {
                code: "DB_DELETE_FAILED".into(),
                message: format!("Failed to delete profile: {}", e),
                action: "Verify profile ID.".into(),
            })?;
        Ok(())
    }

    pub fn get_profile(&self, id: &str) -> Result<Option<DownloadProfile>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, quality_policy, format_policy, destination_directory, naming_template, created_at, updated_at
                 FROM profiles WHERE id = ?1;",
            )
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to prepare profile query: {}", e),
                action: "Retry operation.".into(),
            })?;

        let profile = stmt
            .query_row(params![id], |row| {
                Ok(DownloadProfile {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    quality_policy: row.get(2)?,
                    format_policy: row.get(3)?,
                    destination_directory: row.get(4)?,
                    naming_template: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            })
            .optional()
            .map_err(|e| AppError::DatabaseError {
                code: "DB_QUERY_FAILED".into(),
                message: format!("Failed to get profile: {}", e),
                action: "Verify profile ID.".into(),
            })?;

        Ok(profile)
    }

    fn map_download_row(row: &rusqlite::Row) -> rusqlite::Result<DownloadTask> {
        let status_str: String = row.get(7)?;
        Ok(DownloadTask {
            id: row.get(0)?,
            batch_id: row.get(1)?,
            source_url: row.get(2)?,
            provider_id: row.get(3)?,
            source_id: row.get(4)?,
            title: row.get(5)?,
            thumbnail_url: row.get(6)?,
            status: DownloadStatus::from_str(&status_str),
            priority: row.get(8)?,
            queue_position: row.get(9)?,
            selected_format_id: row.get(10)?,
            selected_quality: row.get(11)?,
            selected_extension: row.get(12)?,
            output_directory: row.get(13)?,
            output_filename: row.get(14)?,
            output_path: row.get(15)?,
            partial_path: row.get(16)?,
            bytes_downloaded: row.get(17)?,
            total_bytes: row.get(18)?,
            progress: row.get(19)?,
            speed_bytes_per_second: row.get(20)?,
            eta_seconds: row.get(21)?,
            retry_count: row.get(22)?,
            max_retries: row.get(23)?,
            next_retry_at: row.get(24)?,
            last_error_code: row.get(25)?,
            last_error_message: row.get(26)?,
            created_at: row.get(27)?,
            started_at: row.get(28)?,
            completed_at: row.get(29)?,
            updated_at: row.get(30)?,
        })
    }
}
