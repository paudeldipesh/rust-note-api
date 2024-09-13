use super::insertables::{GenerateOTPInsertable, NewUser};
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
            username: msg.username,
            email: msg.email,
            password: msg.password,
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
            .first::<User>(&mut connection)
    }
}

impl Handler<GenerateOTPMessage> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: GenerateOTPMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Generate OTP: Unable to establish connection");

        diesel::update(users.filter(email.eq(&msg.email)))
            .set(GenerateOTPInsertable {
                opt_verified: msg.opt_verified,
                opt_enabled: msg.opt_enabled,
                opt_base32: msg.opt_base32,
                opt_auth_url: msg.opt_auth_url,
            })
            .get_result::<User>(&mut connection)
    }
}

impl Handler<VerifyOTPMessage> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: VerifyOTPMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Verify OTP: Unable to establish connection");

        diesel::update(users.filter(email.eq(&msg.email)))
            .set(GenerateOTPInsertable {
                opt_verified: msg.opt_verified,
                opt_enabled: msg.opt_enabled,
                opt_base32: msg.opt_base32,
                opt_auth_url: msg.opt_auth_url,
            })
            .get_result::<User>(&mut connection)
    }
}
