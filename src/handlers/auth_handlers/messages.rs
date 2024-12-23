use crate::models::User;
use actix::Message;
use diesel::QueryResult;

#[derive(Message)]
#[rtype(result = "QueryResult<Vec<User>>")]
pub struct FetchUser;

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[allow(dead_code)]
#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct LoginAndGetUser {
    pub email: String,
    pub password: String,
}

#[derive(Message)]
#[rtype(result = "Result<(usize), diesel::result::Error>")]
pub struct UpdateUserPassword {
    pub user_id: i32,
    pub new_password: String,
}

#[derive(Message)]
#[rtype(result = "Result<usize, diesel::result::Error>")]
pub struct DeleteUser {
    pub user_id: i32,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct OTPMessage {
    pub email: String,
    pub otp_verified: bool,
    pub otp_enabled: bool,
    pub otp_base32: Option<String>,
    pub otp_auth_url: Option<String>,
}
