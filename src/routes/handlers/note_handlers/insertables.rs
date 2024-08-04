use crate::schema::notes;
use chrono::{DateTime, Utc};
use diesel::Insertable;
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=notes)]
pub struct NewNote {
    pub title: String,
    pub content: String,
    pub created_by: i32,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
}
