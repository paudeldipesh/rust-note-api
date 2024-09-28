use crate::routes::{
    handlers::transaction_handlers::transaction_handlers::*,
    middlewares::auth_middlewares::check_auth_middleware,
};
use actix_web::web;
use actix_web_lab::middleware::from_fn;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(
        web::scope("/transaction")
            .wrap(from_fn(check_auth_middleware))
            .service(buy_information),
    );
}
