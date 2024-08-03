use crate::routes::handlers::auth_handlers::*;
use actix_web::web;
use auth_handlers::{login_user, register_user};

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(
        web::scope("/user")
            .service(register_user)
            .service(login_user),
    );
}
