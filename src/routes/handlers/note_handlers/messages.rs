use crate::models::{Note, User};
use actix::Message;
use chrono::offset::Utc;
use chrono::DateTime;
use diesel::QueryResult;
use utoipa::ToSchema;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<User>>")]
pub struct FetchUser;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Note>>")]
pub struct FetchUserNotes {
    pub user_id: i32,
}

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<Note>>")]
pub struct FetchAllNotes;

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
