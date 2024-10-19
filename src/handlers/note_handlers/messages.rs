use crate::models::Note;
use actix::Message;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::QueryResult;
use utoipa::ToSchema;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Note>>")]
pub struct FetchUserNotes {
    pub user_id: i32,
}

#[derive(Message)]
#[rtype(result = "QueryResult<(i64, Vec<Note>, i64, i64)>")]
pub struct FetchNotes {
    pub search: Option<String>,
    pub sort_field: Option<String>,
    pub sort_order: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<Note>")]
pub struct CreateNote {
    pub title: String,
    pub content: String,
    pub created_by: i32,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
}

#[derive(Message)]
#[rtype(result = "QueryResult<Note>")]
pub struct UpdateNote {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_by: i32,
    pub updated_on: DateTime<Utc>,
}

#[derive(Message)]
#[rtype(result = "Result<usize, diesel::result::Error>")]
pub struct DeleteNote {
    pub note_id: i32,
}
