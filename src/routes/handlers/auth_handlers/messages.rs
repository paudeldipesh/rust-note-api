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
pub struct LoginUser {
    pub email: String,
    pub _password: String,
}
