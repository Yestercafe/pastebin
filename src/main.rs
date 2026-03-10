mod db;
mod handlers;
mod models;
mod template;

use actix_web::{web, App, HttpServer};
use std::path::PathBuf;
use std::sync::Arc;

use handlers::AppState;

/// 相对路径时先按当前目录解析，不存在则按可执行文件所在目录解析。
fn resolve_dir(path: String, default_subdir: &str) -> PathBuf {
    let p = PathBuf::from(&path);
    if p.is_absolute() {
        return p;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let from_cwd = cwd.join(&p);
    if from_cwd.exists() {
        return from_cwd;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let from_exe = parent.join(&p);
            if from_exe.exists() {
                return from_exe;
            }
            let default = parent.join(default_subdir);
            if default.exists() {
                return default;
            }
        }
    }
    from_cwd
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_path = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://pastebin.db".to_string());
    let pool = db::init_pool(&db_path)
        .await
        .expect("Failed to create database pool");

    let templates_dir = resolve_dir(
        std::env::var("TEMPLATES_DIR").unwrap_or_else(|_| "templates".to_string()),
        "templates",
    );
    let data_dir = std::env::var("DATA_DIR").unwrap_or_else(|_| "data".to_string());
    let data_dir = resolve_dir(data_dir.clone(), "data");
    std::fs::create_dir_all(&data_dir).ok();

    let static_dir = resolve_dir(
        std::env::var("STATIC_DIR").unwrap_or_else(|_| "static".to_string()),
        "static",
    );

    let static_dir_arc = Arc::new(static_dir);
    let state = web::Data::new(AppState {
        pool,
        templates_dir,
        data_dir,
    });

    HttpServer::new(move || {
        let static_path = (*static_dir_arc).clone();
        App::new()
            .app_data(state.clone())
            .service(handlers::index)
            .service(handlers::list)
            .service(handlers::create_paste)
            .service(handlers::view)
            .service(handlers::edit_form)
            .service(handlers::edit_submit)
            .service(handlers::delete)
            .service(handlers::file)
            .service(actix_files::Files::new("/static", static_path))
    })
    .bind((
        std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        std::env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080),
    ))?
    .run()
    .await
}
