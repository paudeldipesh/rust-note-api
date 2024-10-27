use crate::schema::users;
use diesel::{prelude::AsChangeset, Insertable};
use serde::Serialize;

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name=users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Insertable, Serialize, Clone, AsChangeset)]
#[diesel(table_name=users)]
pub struct OTPInfoInsertable {
    pub otp_verified: bool,
    pub otp_enabled: bool,
    pub otp_base32: Option<String>,
    pub otp_auth_url: Option<String>,
}
