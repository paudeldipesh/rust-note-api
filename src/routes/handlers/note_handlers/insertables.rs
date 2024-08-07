use crate::schema::notes;
use chrono::{DateTime, Utc};
use diesel::Insertable;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Insertable, Serialize, Clone, ToSchema)]
#[diesel(table_name=notes)]
pub struct NewNote {
    #[schema(example = "My note")]
    pub title: String,
    #[schema(example = "this is a note")]
    pub content: String,
    #[schema(example = 1)]
    pub created_by: i32,
    #[schema(example = "2023-09-16T14:35:10.312312")]
    pub created_on: DateTime<Utc>,
    #[schema(example = "2023-09-16T14:38:10.312312")]
    pub updated_on: DateTime<Utc>,
}
