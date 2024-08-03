use crate::models::User;
use actix::Message;
use diesel::QueryResult;

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct CreateUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

#[derive(Message)]
#[rtype(result = "QueryResult<User>")]
pub struct LoginUser {
    pub email: String,
    pub username: String,
}
