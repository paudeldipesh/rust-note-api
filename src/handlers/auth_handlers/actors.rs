use super::insertables::{NewUser, OTPInfoInsertable};
use super::messages::*;
use crate::models::User;
use crate::schema::users::dsl::*;
use crate::utils::db::DbActor;
use actix::Handler;
use diesel::prelude::*;

impl Handler<FetchUser> for DbActor {
    type Result = QueryResult<Vec<User>>;

    fn handle(&mut self, _msg: FetchUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Fetch User: Unable to establish connection");

        users.get_results::<User>(&mut connection)
    }
}

impl Handler<CreateUser> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: CreateUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Create User: Unable to establish connection");

        let new_user: NewUser = NewUser {
            username: msg.username,
            email: msg.email,
            password: msg.password,
        };

        diesel::insert_into(users)
            .values(new_user)
            .get_result::<User>(&mut connection)
    }
}

impl Handler<LoginAndGetUser> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: LoginAndGetUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Login User: Unable to establish connection");

        users
            .filter(email.eq(&msg.email))
            .first::<User>(&mut connection)
    }
}

impl Handler<OTPMessage> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: OTPMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Generate OTP: Unable to establish connection");

        diesel::update(users.filter(email.eq(&msg.email)))
            .set(OTPInfoInsertable {
                opt_verified: msg.opt_verified,
                opt_enabled: msg.opt_enabled,
                opt_base32: msg.opt_base32,
                opt_auth_url: msg.opt_auth_url,
            })
            .get_result::<User>(&mut connection)
    }
}
