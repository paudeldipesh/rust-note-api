use crate::schema::users;
use diesel::Insertable;
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=users)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}
