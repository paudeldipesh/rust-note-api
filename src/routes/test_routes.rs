use super::handlers;
use actix_web::web;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(
        web::scope("/test")
            .service(handlers::test_handlers::index)
            .service(handlers::test_handlers::hello),
    );
}
