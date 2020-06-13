use crate::schema::chapters;

#[derive(Identifiable, Queryable)]
pub struct Chapter {
    pub id: i32,
    pub relative_path: String,
    pub visit_count: i64,
}

#[derive(Queryable)]
pub struct Visits {
    pub id: i64,
    pub chapter_id: i32,
    pub timestamp: i64,
}
