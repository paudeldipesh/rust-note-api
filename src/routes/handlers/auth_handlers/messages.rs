use crate::models::User;
use actix::Message;
use diesel::QueryResult;
use utoipa::ToSchema;

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<User>")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<User>")]
pub struct LoginAndGetUser {
    pub email: String,
    pub _password: String,
}

#[derive(Message, ToSchema)]
#[rtype(result = "QueryResult<User>")]
pub struct LogoutUser {
    pub email: String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct GenerateAndDisableOTPMessage {
    pub email: String,
    pub opt_verified: bool,
    pub opt_enabled: bool,
    pub opt_base32: String,
    pub opt_auth_url: String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct VerifyOTPMessage {
    pub email: String,
    pub opt_verified: bool,
    pub opt_enabled: bool,
    pub opt_base32: String,
    pub opt_auth_url: String,
}
