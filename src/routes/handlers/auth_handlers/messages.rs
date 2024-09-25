use crate::models::User;
use actix::Message;
use diesel::QueryResult;
use utoipa::ToSchema;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<User>>")]
pub struct FetchUser;

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<User>")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<User>")]
pub struct LoginAndGetUser {
    pub email: String,
    pub password: String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct OTPMessage {
    pub email: String,
    pub opt_verified: bool,
    pub opt_enabled: bool,
    pub opt_base32: String,
    pub opt_auth_url: String,
}
