use chrono::{DateTime, Utc};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use crate::models::{Attachment, Paste};

const MAX_CONTENT_LEN: usize = 512 * 1024; // 512KB
const PASTE_ID_LEN: usize = 8;

pub fn generate_paste_id() -> String {
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let rng = uuid::Uuid::new_v4();
    let bytes = rng.as_bytes();
    (0..PASTE_ID_LEN)
        .map(|i| CHARS[bytes[i % 16] as usize % CHARS.len()] as char)
        .collect()
}

pub async fn init_pool(db_path: &str) -> Result<SqlitePool, sqlx::Error> {
    let opts = SqliteConnectOptions::from_str(db_path)?
        .create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .connect_with(opts)
        .await?;
    init_schema(&pool).await?;
    Ok(pool)
}

async fn init_schema(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS pastes (
            id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            title TEXT,
            author TEXT,
            language TEXT,
            expires_at TEXT NOT NULL,
            created_at TEXT NOT NULL,
            user_id INTEGER
        )
        "#,
    )
    .execute(pool)
    .await?;

    // 兼容旧库：若 pastes 表缺少 title 列则补上
    let has_title: Option<(i64,)> = sqlx::query_as(
        "SELECT 1 FROM pragma_table_info('pastes') WHERE name = 'title'",
    )
    .fetch_optional(pool)
    .await?;
    if has_title.is_none() {
        if let Err(e) = sqlx::query("ALTER TABLE pastes ADD COLUMN title TEXT").execute(pool).await {
            log::warn!("db migration add pastes.title: {}", e);
        } else {
            log::info!("db migration: added pastes.title column");
        }
    }

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS attachments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            paste_id TEXT NOT NULL,
            stored_name TEXT NOT NULL,
            original_filename TEXT NOT NULL,
            content_type TEXT NOT NULL,
            is_image INTEGER NOT NULL,
            size INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (paste_id) REFERENCES pastes(id) ON DELETE CASCADE
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub fn content_len_ok(content: &str) -> bool {
    content.len() <= MAX_CONTENT_LEN
}

pub async fn create_paste(
    pool: &SqlitePool,
    id: &str,
    content: &str,
    title: Option<&str>,
    author: Option<&str>,
    language: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO pastes (id, content, title, author, language, expires_at, created_at, user_id)
        VALUES (?, ?, ?, ?, ?, ?, ?, NULL)
        "#,
    )
    .bind(id)
    .bind(content)
    .bind(title)
    .bind(author)
    .bind(language)
    .bind(expires_at.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn paste_exists(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64,)>("SELECT 1 FROM pastes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

pub async fn get_paste(pool: &SqlitePool, id: &str) -> Result<Option<Paste>, sqlx::Error> {
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, String, String, Option<i64>)>(
        "SELECT id, content, title, author, language, expires_at, created_at, user_id FROM pastes WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let expires_at = DateTime::parse_from_rfc3339(&row.5).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
    let created_at = DateTime::parse_from_rfc3339(&row.6).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());

    Ok(Some(Paste {
        id: row.0,
        content: row.1,
        title: row.2,
        author: row.3,
        language: row.4,
        expires_at,
        created_at,
        user_id: row.7,
    }))
}

pub async fn list_pastes(pool: &SqlitePool) -> Result<Vec<Paste>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, String, String, Option<i64>)>(
        "SELECT id, content, title, author, language, expires_at, created_at, user_id FROM pastes ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let expires_at = DateTime::parse_from_rfc3339(&row.5).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
        let created_at = DateTime::parse_from_rfc3339(&row.6).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
        out.push(Paste {
            id: row.0,
            content: row.1,
            title: row.2,
            author: row.3,
            language: row.4,
            expires_at,
            created_at,
            user_id: row.7,
        });
    }
    Ok(out)
}

pub async fn update_paste(
    pool: &SqlitePool,
    id: &str,
    content: &str,
    title: Option<&str>,
    author: Option<&str>,
    language: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE pastes SET content = ?, title = ?, author = ?, language = ?, expires_at = ?
        WHERE id = ?
        "#,
    )
    .bind(content)
    .bind(title)
    .bind(author)
    .bind(language)
    .bind(expires_at.to_rfc3339())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_paste(pool: &SqlitePool, id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM attachments WHERE paste_id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    sqlx::query("DELETE FROM pastes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

// Attachments
pub async fn create_attachment(
    pool: &SqlitePool,
    paste_id: &str,
    stored_name: &str,
    original_filename: &str,
    content_type: &str,
    is_image: bool,
    size: i64,
) -> Result<(), sqlx::Error> {
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO attachments (paste_id, stored_name, original_filename, content_type, is_image, size, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(paste_id)
    .bind(stored_name)
    .bind(original_filename)
    .bind(content_type)
    .bind(if is_image { 1i32 } else { 0i32 })
    .bind(size)
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_attachments(pool: &SqlitePool, paste_id: &str) -> Result<Vec<Attachment>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, String, i32, i64, String)>(
        "SELECT id, paste_id, stored_name, original_filename, content_type, is_image, size, created_at FROM attachments WHERE paste_id = ? ORDER BY created_at",
    )
    .bind(paste_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let created_at = DateTime::parse_from_rfc3339(&r.7).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
            Attachment {
                id: r.0,
                paste_id: r.1,
                stored_name: r.2,
                original_filename: r.3,
                content_type: r.4,
                is_image: r.5 != 0,
                size: r.6,
                created_at,
            }
        })
        .collect())
}

pub async fn get_attachment(
    pool: &SqlitePool,
    paste_id: &str,
    stored_name: &str,
) -> Result<Option<Attachment>, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, String, String, String, String, i32, i64, String)>(
        "SELECT id, paste_id, stored_name, original_filename, content_type, is_image, size, created_at FROM attachments WHERE paste_id = ? AND stored_name = ?",
    )
    .bind(paste_id)
    .bind(stored_name)
    .fetch_optional(pool)
    .await?;

    let r = match row {
        Some(x) => x,
        None => return Ok(None),
    };
    let created_at = DateTime::parse_from_rfc3339(&r.7).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now());
    Ok(Some(Attachment {
        id: r.0,
        paste_id: r.1,
        stored_name: r.2,
        original_filename: r.3,
        content_type: r.4,
        is_image: r.5 != 0,
        size: r.6,
        created_at,
    }))
}
