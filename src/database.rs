use std::convert::TryInto;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use actix::{Actor, Addr, Handler, Message, SyncArbiter, SyncContext};
use diesel::{Connection, insert_into, sql_query};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sql_types::{BigInt, VarChar, Bigint};
use serde::{Serialize, Deserialize};

use dotenv::dotenv;
use indoc::indoc;

use crate::models::Chapter;

fn get_current_timestamp() -> i64 {
    return SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis().try_into().expect("Hello future");
}

pub struct DbExecutor(PgConnection);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub fn get_db_executor() -> Addr<DbExecutor> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    return SyncArbiter::start(4, move || {
        DbExecutor(PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}.", database_url)))
    });
}

pub struct RecordVisit {
    pub relative_path: String,
}

impl Message for RecordVisit {
    type Result = Result<(), Error>;
}

fn get_chapter(connection: &PgConnection, relative_path_value: &str) -> Result<Chapter, Error> {
    use crate::schema::chapters::dsl::*;
    let chapter: Option<Chapter> = chapters
        .filter(relative_path.eq(relative_path_value))
        .first::<Chapter>(connection)
        .optional()?;
    return if let Some(chapter) = chapter {
        Ok(chapter)
    } else {
        let row: Chapter = insert_into(chapters)
            .values(relative_path.eq(relative_path_value))
            .get_result(connection)?;
        Ok(row)
    };
}

fn inc_visit(connection: &PgConnection, chapter: &Chapter) -> Result<(), Error> {
    use crate::schema::chapters::dsl::*;
    diesel::update(chapter)
        .set(visit_count.eq(visit_count + 1))
        .execute(connection)?;
    return Ok(());
}

impl Handler<RecordVisit> for DbExecutor {
    type Result = Result<(), Error>;

    fn handle(&mut self, msg: RecordVisit, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0;
        let chapter = get_chapter(connection, msg.relative_path.as_str())?;
        use crate::schema::visits::dsl::*;
        insert_into(visits)
            .values((
                chapter_id.eq(chapter.id),
                timestamp.eq(get_current_timestamp())
            ))
            .execute(connection)?;
        inc_visit(connection, &chapter)?;
        return Ok(());
    }
}

#[derive(Serialize)]
pub struct OneChapterVisitInfo {
    relative_path: String,
    visit_count: i64,
}

pub type ChapterVisitInfo = Vec<OneChapterVisitInfo>;

pub struct ListChaptersAll {
    pub page: i32,
}

impl Message for ListChaptersAll {
    type Result = Result<ChapterVisitInfo, Error>;
}

const PAGE_SIZE: i32 = 50;

impl Handler<ListChaptersAll> for DbExecutor {
    type Result = Result<ChapterVisitInfo, Error>;

    fn handle(&mut self, msg: ListChaptersAll, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0;
        use crate::schema::chapters::dsl::*;
        let showing_chapters: Vec<Chapter> = chapters
            .order(visit_count.desc())
            .offset(((msg.page - 1) * PAGE_SIZE).into())
            .limit(PAGE_SIZE.into())
            .load::<Chapter>(connection)?;
        let chapter_visit_info = showing_chapters.into_iter().map(|showing_chapter| OneChapterVisitInfo {
            visit_count: showing_chapter.visit_count,
            relative_path: showing_chapter.relative_path,
        }).collect();
        return Ok(chapter_visit_info);
    }
}

#[derive(Deserialize, Copy, Clone)]
pub enum TimeFrame {
    HOUR,
    DAY,
    WEEK,
    MONTH,
    YEAR,
}

impl TimeFrame {
    fn get_milliseconds(&self) -> i64 {
        return match self {
            TimeFrame::HOUR => 1000 * 3600,
            TimeFrame::DAY => 1000 * 3600 * 24,
            TimeFrame::WEEK => 1000 * 3600 * 24 * 7,
            TimeFrame::MONTH => 1000 * 3600 * 24 * 30,
            TimeFrame::YEAR => 1000 * 3600 * 24 * 365,
        };
    }
}

pub struct ListChapterRecent {
    pub page: i32,
    pub time_frame: TimeFrame,
}

impl Message for ListChapterRecent {
    type Result = Result<ChapterVisitInfo, Error>;
}

impl Handler<ListChapterRecent> for DbExecutor {
    type Result = Result<ChapterVisitInfo, Error>;

    fn handle(&mut self, msg: ListChapterRecent, _: &mut Self::Context) -> Self::Result {
        let connection = &self.0;

        #[derive(QueryableByName)]
        struct RecentAggregateResult {
            #[sql_type="VarChar"]
            relative_path: String,
            #[sql_type="BigInt"]
            visit_count: i64,
        }

        let showing_chapters: Vec<RecentAggregateResult> = sql_query(indoc!("
            SELECT chapters.relative_path, count(1) as visit_count FROM visits
                LEFT JOIN chapters
                    ON visits.chapter_id = chapters.id
                WHERE visits.timestamp > $1
                GROUP BY chapters.id
                ORDER BY visit_count DESC
                LIMIT $2
                OFFSET $3
        "))
            .bind::<Bigint, i64>(get_current_timestamp() - msg.time_frame.get_milliseconds())
            .bind::<Bigint, i64>(PAGE_SIZE.into())
            .bind::<Bigint, i64>(((msg.page - 1) * PAGE_SIZE).into())
            .get_results(connection)?;

        let chapter_visit_info = showing_chapters.into_iter().map(|showing_chapter| OneChapterVisitInfo {
            visit_count: showing_chapter.visit_count,
            relative_path: showing_chapter.relative_path,
        }).collect();
        return Ok(chapter_visit_info);
    }
}
