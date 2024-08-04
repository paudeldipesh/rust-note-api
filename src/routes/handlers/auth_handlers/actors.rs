use super::insertables::NewUser;
use super::messages::*;
use crate::models::User;
use crate::schema::users::dsl::*;
use crate::utils::db::DbActor;
use actix::Handler;
use diesel::prelude::*;

impl Handler<CreateUser> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: CreateUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Create User: Unable to establish connection");

        let new_user: NewUser = NewUser {
            first_name: msg.first_name,
            last_name: msg.last_name,
            username: msg.username,
            email: msg.email,
        };

        diesel::insert_into(users)
            .values(new_user)
            .get_result::<User>(&mut connection)
    }
}

impl Handler<LoginUser> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: LoginUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Login User: Unable to establish connection");

        users
            .filter(email.eq(&msg.email))
            .filter(username.eq(&msg.username))
            .first::<User>(&mut connection)
    }
}
