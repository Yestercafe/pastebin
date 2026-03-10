use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use chrono::{Duration, Utc};
use futures_util::TryStreamExt;
use serde::Deserialize;
use sqlx::SqlitePool;
use std::io::Write;
use std::path::PathBuf;

use crate::db;
use crate::template;

const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
const MAX_PASTE_TOTAL_SIZE: usize = 20 * 1024 * 1024; // 20MB

fn expires_at_from_option(opt: &str) -> chrono::DateTime<Utc> {
    let now = Utc::now();
    match opt {
        "1d" => now + Duration::days(1),
        "1w" => now + Duration::weeks(1),
        "1m" => now + Duration::days(30),
        "never" => now + Duration::days(365 * 10),
        _ => now + Duration::days(1),
    }
}

fn is_allowed_mime(content_type: &str) -> bool {
    let ct = content_type.to_lowercase();
    if ct.starts_with("image/") {
        return true;
    }
    let allowed = [
        "application/pdf",
        "text/plain",
        "application/zip",
        "application/x-zip-compressed",
        "application/json",
        "text/csv",
    ];
    allowed.contains(&ct.as_str())
}

fn is_image(content_type: &str) -> bool {
    content_type.to_lowercase().starts_with("image/")
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

pub struct AppState {
    pub pool: SqlitePool,
    pub templates_dir: PathBuf,
    pub data_dir: PathBuf,
}

#[get("/")]
async fn index(state: web::Data<AppState>) -> impl Responder {
    let html = template::render_from_dir(&state.templates_dir, "index.html", ())
        .unwrap_or_else(|e| format!("<pre>Template error: {}</pre>", e));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[derive(serde::Serialize)]
struct ListPasteItem {
    id: String,
    content: String,
    title: Option<String>,
    author: Option<String>,
    language: Option<String>,
    created_at: String,
    expires_at: String,
    expired: bool,
}

#[get("/list")]
async fn list(state: web::Data<AppState>) -> impl Responder {
    let pastes = db::list_pastes(&state.pool).await.unwrap_or_default();
    let now = Utc::now();
    let items: Vec<ListPasteItem> = pastes
        .into_iter()
        .map(|p| {
            let expired = p.expires_at < now;
            ListPasteItem {
                id: p.id,
                content: p.content,
                title: p.title,
                author: p.author,
                language: p.language,
                created_at: p.created_at.format("%Y-%m-%d %H:%M").to_string(),
                expires_at: p.expires_at.format("%Y-%m-%d %H:%M").to_string(),
                expired,
            }
        })
        .collect();
    #[derive(serde::Serialize)]
    struct ListCtx {
        items: Vec<ListPasteItem>,
    }
    let html = template::render_from_dir(&state.templates_dir, "list.html", ListCtx { items })
        .unwrap_or_else(|e| format!("<pre>Template error: {}</pre>", e));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[post("/paste")]
async fn create_paste(state: web::Data<AppState>, mut payload: Multipart) -> impl Responder {
    let mut content = String::new();
    let mut title: Option<String> = None;
    let mut author: Option<String> = None;
    let mut language: Option<String> = None;
    let mut expires = "1d".to_string();
    let mut files: Vec<(String, Vec<u8>, String)> = Vec::new(); // (filename, body, content_type)
    let mut total_size: usize = 0;

    while let Ok(Some(mut field)) = payload.try_next().await {
        let name = field.content_disposition().get_name().unwrap_or("").to_string();
        let ctype = field.content_type().map(|c| c.to_string()).unwrap_or_default();

        if name == "content" {
            while let Ok(Some(chunk)) = field.try_next().await {
                if content.len() + chunk.len() > 512 * 1024 {
                    return HttpResponse::BadRequest().body("Content too long");
                }
                content.push_str(&String::from_utf8_lossy(&chunk));
            }
        } else if name == "title" {
            let mut s = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                s.push_str(&String::from_utf8_lossy(&chunk));
            }
            if !s.is_empty() {
                title = Some(s.trim().to_string());
            }
        } else if name == "author" {
            let mut s = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                s.push_str(&String::from_utf8_lossy(&chunk));
            }
            if !s.is_empty() {
                author = Some(s.trim().to_string());
            }
        } else if name == "language" {
            let mut s = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                s.push_str(&String::from_utf8_lossy(&chunk));
            }
            if !s.is_empty() {
                language = Some(s.trim().to_string());
            }
        } else if name == "expires" {
            let mut s = String::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                s.push_str(&String::from_utf8_lossy(&chunk));
            }
            if !s.is_empty() {
                expires = s.trim().to_string();
            }
        } else if name == "files" || name == "file" {
            let filename = field
                .content_disposition()
                .get_filename()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "unnamed".to_string());
            let mut body = Vec::new();
            while let Ok(Some(chunk)) = field.try_next().await {
                if body.len() + chunk.len() > MAX_FILE_SIZE {
                    return HttpResponse::BadRequest().body("File too large");
                }
                body.extend_from_slice(&chunk);
            }
            if !body.is_empty() && is_allowed_mime(&ctype) {
                total_size += body.len();
                if total_size > MAX_PASTE_TOTAL_SIZE {
                    return HttpResponse::BadRequest().body("Total upload size exceeded");
                }
                files.push((filename, body, ctype));
            }
        }
    }

    if content.is_empty() {
        return HttpResponse::BadRequest().body("Content is required");
    }

    let mut id = db::generate_paste_id();
    while db::paste_exists(&state.pool, &id).await.unwrap_or(false) {
        id = db::generate_paste_id();
    }

    let expires_at = expires_at_from_option(&expires);
    if let Err(e) = db::create_paste(
        &state.pool,
        &id,
        &content,
        title.as_deref(),
        author.as_deref(),
        language.as_deref(),
        expires_at,
    )
    .await
    {
        return HttpResponse::InternalServerError().body(format!("DB error: {}", e));
    }

    let paste_dir = state.data_dir.join(&id);
    if std::fs::create_dir_all(&paste_dir).is_err() {
        let _ = db::delete_paste(&state.pool, &id).await;
        return HttpResponse::InternalServerError().body("Failed to create upload directory");
    }

    for (original_name, body, content_type) in files {
        let path = PathBuf::from(&original_name);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let stored_name = format!("{}.{}", uuid::Uuid::new_v4().simple(), ext);
        let path = paste_dir.join(&stored_name);
        if let Ok(mut f) = std::fs::File::create(&path) {
            let _ = f.write_all(&body);
            let is_image = is_image(&content_type);
            let _ = db::create_attachment(
                &state.pool,
                &id,
                &stored_name,
                &original_name,
                &content_type,
                is_image,
                body.len() as i64,
            )
            .await;
        }
    }

    HttpResponse::SeeOther()
        .append_header(("Location", format!("/p/{}", id)))
        .finish()
}

#[get("/p/{id}")]
async fn view(state: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let id = id.into_inner();
    let paste = match db::get_paste(&state.pool, &id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().body("Paste not found"),
        Err(_) => return HttpResponse::InternalServerError().body("DB error"),
    };

    if paste.expires_at < Utc::now() {
        return HttpResponse::Gone().body("Paste has expired");
    }

    let attachments = db::list_attachments(&state.pool, &id).await.unwrap_or_default();
    #[derive(serde::Serialize)]
    struct ViewPaste {
        id: String,
        content: String,
        title: Option<String>,
        author: Option<String>,
        language: Option<String>,
        created_at: String,
        expires_at: String,
    }
    #[derive(serde::Serialize)]
    struct ViewAttach {
        id: i64,
        paste_id: String,
        stored_name: String,
        original_filename: String,
        content_type: String,
        is_image: bool,
        size: i64,
    }
    let view_paste = ViewPaste {
        id: paste.id.clone(),
        content: html_escape(&paste.content),
        title: paste.title.clone(),
        author: paste.author.clone(),
        language: paste.language.clone(),
        created_at: paste.created_at.format("%Y-%m-%d %H:%M").to_string(),
        expires_at: paste.expires_at.format("%Y-%m-%d %H:%M").to_string(),
    };
    let view_attachments: Vec<ViewAttach> = attachments
        .iter()
        .map(|a| ViewAttach {
            id: a.id,
            paste_id: a.paste_id.clone(),
            stored_name: a.stored_name.clone(),
            original_filename: a.original_filename.clone(),
            content_type: a.content_type.clone(),
            is_image: a.is_image,
            size: a.size,
        })
        .collect();
    #[derive(serde::Serialize)]
    struct ViewCtx {
        paste: ViewPaste,
        attachments: Vec<ViewAttach>,
    }
    let html = template::render_from_dir(
        &state.templates_dir,
        "view.html",
        ViewCtx { paste: view_paste, attachments: view_attachments },
    )
    .unwrap_or_else(|e| format!("<pre>Template error: {}</pre>", e));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[get("/p/{id}/edit")]
async fn edit_form(state: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let id = id.into_inner();
    let paste = match db::get_paste(&state.pool, &id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().body("Paste not found"),
        Err(_) => return HttpResponse::InternalServerError().body("DB error"),
    };
    let attachments = db::list_attachments(&state.pool, &id).await.unwrap_or_default();
    #[derive(serde::Serialize)]
    struct EditPaste {
        id: String,
        content: String,
        title: Option<String>,
        author: Option<String>,
        language: Option<String>,
        created_at: String,
        expires_at: String,
    }
    #[derive(serde::Serialize)]
    struct EditAttach {
        id: i64,
        stored_name: String,
        original_filename: String,
    }
    let edit_paste = EditPaste {
        id: paste.id.clone(),
        content: paste.content.clone(),
        title: paste.title.clone(),
        author: paste.author.clone(),
        language: paste.language.clone(),
        created_at: paste.created_at.format("%Y-%m-%d %H:%M").to_string(),
        expires_at: paste.expires_at.format("%Y-%m-%d %H:%M").to_string(),
    };
    let edit_attachments: Vec<EditAttach> = attachments
        .iter()
        .map(|a| EditAttach { id: a.id, stored_name: a.stored_name.clone(), original_filename: a.original_filename.clone() })
        .collect();
    #[derive(serde::Serialize)]
    struct EditCtx {
        paste: EditPaste,
        attachments: Vec<EditAttach>,
    }
    let html = template::render_from_dir(
        &state.templates_dir,
        "edit.html",
        EditCtx { paste: edit_paste, attachments: edit_attachments },
    )
    .unwrap_or_else(|e| format!("<pre>Template error: {}</pre>", e));
    HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html)
}

#[derive(Debug, Deserialize)]
struct EditForm {
    content: String,
    title: Option<String>,
    author: Option<String>,
    language: Option<String>,
    expires: Option<String>,
}

#[post("/p/{id}/edit")]
async fn edit_submit(
    state: web::Data<AppState>,
    id: web::Path<String>,
    form: web::Form<EditForm>,
) -> impl Responder {
    let id = id.into_inner();
    let paste = match db::get_paste(&state.pool, &id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().body("Not found"),
        Err(_) => return HttpResponse::InternalServerError().body("DB error"),
    };

    let content = if form.content.is_empty() { &paste.content } else { &form.content };
    let title = form.title.as_deref().filter(|s| !s.is_empty());
    let author = form.author.as_deref().filter(|s| !s.is_empty());
    let language = form.language.as_deref().filter(|s| !s.is_empty());
    let expires_opt = form.expires.as_deref().unwrap_or("1d");
    let expires_at = expires_at_from_option(expires_opt);

    if !db::content_len_ok(content) {
        return HttpResponse::BadRequest().body("Content too long");
    }

    if db::update_paste(
        &state.pool,
        &id,
        content,
        title,
        author,
        language,
        expires_at,
    )
    .await
    .is_err()
    {
        return HttpResponse::InternalServerError().body("Update failed");
    }

    HttpResponse::SeeOther()
        .append_header(("Location", format!("/p/{}", id)))
        .finish()
}

#[post("/p/{id}/delete")]
async fn delete(state: web::Data<AppState>, id: web::Path<String>) -> impl Responder {
    let id = id.into_inner();
    if db::get_paste(&state.pool, &id).await.ok().flatten().is_none() {
        return HttpResponse::NotFound().finish();
    }
    let _ = db::delete_paste(&state.pool, &id).await;
    let dir = state.data_dir.join(&id);
    let _ = std::fs::remove_dir_all(&dir);
    HttpResponse::SeeOther()
        .append_header(("Location", "/list"))
        .finish()
}

#[get("/p/{id}/file/{stored_name}")]
async fn file(
    state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> impl Responder {
    let (id, stored_name) = path.into_inner();
    let paste = match db::get_paste(&state.pool, &id).await {
        Ok(Some(p)) => p,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    if paste.expires_at < Utc::now() {
        return HttpResponse::Gone().finish();
    }
    let att = match db::get_attachment(&state.pool, &id, &stored_name).await {
        Ok(Some(a)) => a,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let file_path = state.data_dir.join(&id).join(&stored_name);
    let body = match std::fs::read(&file_path) {
        Ok(b) => b,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let disp = if att.is_image {
        "inline".to_string()
    } else {
        format!("attachment; filename=\"{}\"", att.original_filename.replace('"', "%22"))
    };
    let content_type = att.content_type.parse::<mime::Mime>().unwrap_or(mime::APPLICATION_OCTET_STREAM);
    HttpResponse::Ok()
        .content_type(content_type)
        .insert_header(("Content-Disposition", disp))
        .body(body)
}
