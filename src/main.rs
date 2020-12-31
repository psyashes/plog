use actix_web::{get, App, HttpResponse, HttpServer, ResponseError};
use thiserror::Error;
use askama::Template;
use chrono::{Utc, Local, DateTime, Date};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;

#[derive(Error, Debug)]
enum MyError {
    #[error("Failed to render HTML")]
    AskamaError(#[from] askama::Error),
}
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    entries: Vec<ProgressLog>,
}

impl ResponseError for MyError {}

struct ProgressLog {
    id: u32,
    text: String,
    created_at: DateTime<Local>,
}

#[get("/")]
async fn index() -> Result<HttpResponse, MyError> {
    let mut entries = Vec::new();
    entries.push(ProgressLog {
        id: 1,
        text: "My progress log".to_string(),
        created_at: Local::now(),
    });
    entries.push(ProgressLog {
        id: 2,
        text: "I did some tasks.".to_string(),
        created_at: Local::now(),
    });
    let html = IndexTemplate { entries };
    let response_body = html.render()?;
    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(response_body))
}

#[actix_rt::main]
async fn main() -> Result<(), actix_web::Error> {
    let manager = SqliteConnectionManager::file("todo.db");
    let pool = Pool::new(manager).expect("Failed to initialize the connection pool.");
    let conn = pool
        .get()
        .expect("Failed to get the connection from the pool.");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS todo (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            text TEXT NOT NULL,
            created_at TEXT NOT NULL,
        )",
        params![],
    )
    .expect("Failed to create a table `todo`.");

    HttpServer::new(move || App::new().service(index))
        .bind("0.0.0.0:8080")?
        .run()
        .await?;
    Ok(())
}
