use crate::models::Note;
use actix::Message;
use chrono::NaiveDateTime;
use diesel::QueryResult;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Clone, Copy)]
pub enum ActiveStatus {
    Active,
    Inactive,
}

impl ActiveStatus {
    pub fn as_bool(self) -> bool {
        match self {
            ActiveStatus::Active => true,
            ActiveStatus::Inactive => false,
        }
    }
}

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
    pub active_status: Option<ActiveStatus>,
}

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<Note>")]
pub struct CreateNote {
    pub title: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_by: i32,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
}

#[derive(Message)]
#[rtype(result = "QueryResult<Note>")]
pub struct UpdateNote {
    pub id: i32,
    pub title: String,
    pub _image_url: Option<String>,
    pub content: String,
    pub created_by: i32,
    pub active: bool,
    pub updated_on: NaiveDateTime,
}

#[derive(Message)]
#[rtype(result = "Result<usize, diesel::result::Error>")]
pub struct DeleteNote {
    pub note_id: i32,
}
