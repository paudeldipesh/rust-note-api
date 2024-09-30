use crate::{
    handlers::transaction_handlers::transaction_handlers::*,
    middlewares::auth_middlewares::check_auth_middleware,
};
use actix_web::web;
use actix_web_lab::middleware::from_fn;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure
        .service(
            web::scope("/transaction")
                .service(get_buy_quote)
                .service(get_buy_information)
                .service(get_swap_transaction),
        )
        .service(
            web::scope("/secure/transaction")
                .wrap(from_fn(check_auth_middleware))
                .service(get_buy_lists),
        );
}
