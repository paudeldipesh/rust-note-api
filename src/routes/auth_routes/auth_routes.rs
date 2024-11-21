use crate::handlers::auth_handlers::{two_fa_handlers::*, user_handlers::*};
use crate::{handlers::auth_handlers::*, middlewares::auth_middlewares::*};
use actix_web::web;
use actix_web_lab::middleware::from_fn;
use auth_handlers::*;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure
        .service(
            web::scope("/user")
                .service(register_user)
                .service(login_user)
                .service(logout_user),
        )
        .service(
            web::scope("/auth")
                .wrap(from_fn(check_auth_middleware))
                .service(generate_otp_handler)
                .service(verify_otp_handler)
                .service(token_validate_handler)
                .service(disable_otp_handler)
                .service(get_user)
                .service(delete_user)
                .service(update_password),
        );
}
