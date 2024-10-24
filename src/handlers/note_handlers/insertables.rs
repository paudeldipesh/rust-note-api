use crate::schema::notes;
use chrono::NaiveDateTime;
use diesel::Insertable;
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=notes)]
pub struct NewNote {
    pub title: String,
    pub content: String,
    pub image_url: Option<String>,
    pub created_by: i32,
    pub created_on: NaiveDateTime,
    pub updated_on: NaiveDateTime,
}
