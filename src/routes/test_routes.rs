use super::handlers;
use actix_web::web;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(handlers::test_handlers::home).service(
        web::scope("/test")
            .service(handlers::test_handlers::index)
            .service(handlers::test_handlers::hello),
    );
}
