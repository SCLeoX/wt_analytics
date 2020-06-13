#[macro_use]
extern crate diesel;

use actix::Addr;
use actix_web::{App, FromRequest, HttpResponse, HttpServer, Responder, web, HttpRequest};
use serde::Deserialize;

use database::DbExecutor;
use error::WTError;

use crate::database::{get_db_executor, ListChaptersAll, RecordVisit, TimeFrame, ListChapterRecent};
use crate::allowed_origin::is_allowed_origin;

pub mod schema;
mod models;
mod database;
mod error;
mod allowed_origin;

async fn count(req: HttpRequest, state: web::Data<AppState>, content: String) -> Result<impl Responder, WTError> {
    if let Some(origin) = req.headers().get("origin").map(|header| header.to_str().ok()).flatten() {
        if is_allowed_origin(origin) {
            state.db.send(RecordVisit { relative_path: content }).await??;
            return Ok(HttpResponse::Ok()
                .body("<3")
                .with_header("Access-Control-Allow-Origin", origin)
                .with_header("Vary", "Origin")
            );
        }
    }
    return Ok(HttpResponse::Forbidden()
        .body("POST /count may only be accessed via whitelisted origins.")
        .with_header("Vary", "Origin")
    );
}

#[derive(Deserialize)]
struct ChapterAllQuery {
    page: i32,
}
async fn chapter_all(state: web::Data<AppState>, query: web::Query<ChapterAllQuery>) -> Result<impl Responder, WTError> {
    let result = state.db.send(ListChaptersAll { page: query.page }).await??;
    return Ok(HttpResponse::Ok().json(result).with_header("Access-Control-Allow-Origin", "*"));
}

#[derive(Deserialize)]
struct ChapterRecentQuery {
    page: i32,
    time_frame: TimeFrame,
}
async fn chapter_recent(state: web::Data<AppState>, query: web::Query<ChapterRecentQuery>) -> Result<impl Responder, WTError> {
    let result = state.db.send(ListChapterRecent { page: query.page, time_frame: query.time_frame }).await??;
    return Ok(HttpResponse::Ok().json(result).with_header("Access-Control-Allow-Origin", "*"));
}

struct AppState {
    db: Addr<DbExecutor>
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let db_addr = get_db_executor();
    HttpServer::new(move || {
        App::new()
            .data(AppState {
                db: db_addr.clone(),
            })
            .service(
                web::resource("/count")
                    .app_data(String::configure(|cfg| {
                        cfg.limit(1024)
                    }))
                    .route(web::post().to(count))
            )
            .route(
                "/api/chapters/all",
                web::get().to(chapter_all),
            )
            .route(
                "/api/chapters/recent",
                web::get().to(chapter_recent),
            )
    })
        .bind("127.0.0.1:8088")?
        .run()
        .await
}
