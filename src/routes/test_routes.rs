use super::handlers::test_handlers::*;
use actix_web::web;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure
        .service(home)
        .service(web::scope("/test").service(index).service(hello));
}
