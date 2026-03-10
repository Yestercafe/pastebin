use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Paste {
    pub id: String,
    pub content: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Attachment {
    pub id: i64,
    pub paste_id: String,
    pub stored_name: String,
    pub original_filename: String,
    pub content_type: String,
    pub is_image: bool,
    pub size: i64,
    pub created_at: DateTime<Utc>,
}
