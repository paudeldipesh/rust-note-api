use crate::{fetch_notes, fetch_users, middlewares::auth_middlewares::auth_stats_middleware};
use actix_web::web;
use actix_web_lab::middleware::from_fn;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(
        web::scope("/admin/dashboard")
            .wrap(from_fn(auth_stats_middleware))
            .service(fetch_notes)
            .service(fetch_users),
    );
}
