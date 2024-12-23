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

        let user_count: i64 = users
            .count()
            .get_result::<i64>(&mut connection)
            .expect("Unable to get user count");

        let user_role: String = if user_count == 0 {
            String::from("admin")
        } else {
            String::from("user")
        };

        let new_user: NewUser = NewUser {
            username: msg.username,
            email: msg.email,
            password: msg.password,
            role: user_role,
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
            .expect("Login And Get User: Unable to establish connection");

        users
            .filter(email.eq(&msg.email))
            .first::<User>(&mut connection)
    }
}

impl Handler<UpdateUserPassword> for DbActor {
    type Result = Result<usize, diesel::result::Error>;

    fn handle(&mut self, msg: UpdateUserPassword, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Update User Password: Unable to establish connection");

        diesel::update(users.filter(id.eq(msg.user_id)))
            .set(password.eq(msg.new_password))
            .execute(&mut connection)
    }
}

impl Handler<DeleteUser> for DbActor {
    type Result = Result<usize, diesel::result::Error>;

    fn handle(&mut self, msg: DeleteUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Delete User: Unable to establish connection");

        diesel::delete(users.filter(id.eq(msg.user_id))).execute(&mut connection)
    }
}

impl Handler<OTPMessage> for DbActor {
    type Result = QueryResult<User>;

    fn handle(&mut self, msg: OTPMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("OTP Message: Unable to establish connection");

        diesel::update(users.filter(email.eq(&msg.email)))
            .set(OTPInfoInsertable {
                otp_verified: msg.otp_verified,
                otp_enabled: msg.otp_enabled,
                otp_base32: msg.otp_base32,
                otp_auth_url: msg.otp_auth_url,
            })
            .get_result::<User>(&mut connection)
    }
}
