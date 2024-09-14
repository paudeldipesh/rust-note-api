use crate::schema::users;
use diesel::{prelude::AsChangeset, Insertable};
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Insertable, Serialize, Clone, AsChangeset)]
#[diesel(table_name=users)]
pub struct OTPInfoInsertable {
    pub opt_verified: bool,
    pub opt_enabled: bool,
    pub opt_base32: String,
    pub opt_auth_url: String,
}
