table! {
    chapters (id) {
        id -> Int4,
        relative_path -> Varchar,
        visit_count -> Int8,
    }
}

table! {
    visits (id) {
        id -> Int8,
        chapter_id -> Int4,
        timestamp -> Int8,
    }
}

joinable!(visits -> chapters (chapter_id));

allow_tables_to_appear_in_same_query!(
    chapters,
    visits,
);
