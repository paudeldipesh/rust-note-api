use crate::models::User;
use actix::Message;
use diesel::QueryResult;

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct LoginUser {
    pub email: String,
    pub _password: String,
}
