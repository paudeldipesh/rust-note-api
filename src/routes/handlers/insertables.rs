use crate::schema::notes;
use diesel::Insertable;
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=notes)]
pub struct NewNote {
    pub title: String,
    pub content: String,
    pub created_by: i32,
}
