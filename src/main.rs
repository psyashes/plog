use actix_web::{get, http::header, post, web, App, HttpResponse, HttpServer, ResponseError};
use thiserror::Error;
use askama::Template;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::Deserialize;
use chrono::{Local, DateTime};

#[derive(Deserialize)]
struct AddParams {
    text: String,
    created_at: String,
}

#[derive(Deserialize)]
struct DeleteParams {
    id: u32,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<LogEntry>,
}

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),

    #[error("Failed to get connection")]
    ConnectionPoolError(#[from] r2d2::Error),

    #[error("Failed SQL executtion")]
    SQLiteError(#[from] rusqlite::Error),
}

impl ResponseError for MyError {}

struct LogEntry {
    id: u32,
    text: String,
    created_at: String,
}

#[post("/add")]
async fn add_log(
    params: web::Form<AddParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    let now = String::from(Local::now().to_string());
    conn.execute("INSERT INTO progress_logs (text, created_at) VALUES (?, ?)", &[&params.text, &now])?;
    Ok(HttpResponse::SeeOther()
       .header(header::LOCATION, "/")
       .finish())
}

#[post("/delete")]
async fn delete_log(
    params: web::Form<DeleteParams>,
    db: web::Data<r2d2::Pool<SqliteConnectionManager>>,
) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    conn.execute("DELETE FROM progress_logs WHERE id=?", &[&params.id])?;
    Ok(HttpResponse::SeeOther()
       .header(header::LOCATION, "/")
       .finish())
}

#[get("/")]
async fn index(db: web::Data<Pool<SqliteConnectionManager>>) -> Result<HttpResponse, MyError> {
    let conn = db.get()?;
    let mut statement = conn.prepare("SELECT id, text, created_at FROM progress_logs")?;
    let rows = statement.query_map(params![], |row| {
        let id = row.get(0)?;
        let text = row.get(1)?;
        let created_at = row.get(2)?;
        Ok(LogEntry { id, text, created_at })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    let html = IndexTemplate { entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(response_body))
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let manager = SqliteConnectionManager::file("progress_log.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let conn = pool
        .get()
        .expect("Failed to get the connection from the pool.");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS progress_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        params![],
    )
    .expect("Failed to create a table `progress_logs`.");

    HttpServer::new(move || App::new().service(index).data(pool.clone()))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
