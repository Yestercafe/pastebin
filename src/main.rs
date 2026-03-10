mod config;
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
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cfg = config::load(&config::config_path());

    let pool = db::init_pool(&cfg.database_url)
        .await
        .expect("Failed to create database pool");

    let templates_dir = resolve_dir(cfg.templates_dir.clone(), "templates");
    let data_dir = resolve_dir(cfg.data_dir.clone(), "data");
    std::fs::create_dir_all(&data_dir).ok();

    let static_dir = resolve_dir(cfg.static_dir.clone(), "static");

    log::info!("database: {}", cfg.database_url);
    log::info!("data_dir: {}", data_dir.display());
    log::info!("templates_dir: {}", templates_dir.display());

    let static_dir_arc = Arc::new(static_dir);
    let state = web::Data::new(AppState {
        pool,
        templates_dir,
        data_dir,
    });

    let host = cfg.host.clone();
    let port = cfg.port;
    log::info!("listening on http://{}:{}", host, port);

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
    .bind((host.clone(), port))?
    .run()
    .await
}
