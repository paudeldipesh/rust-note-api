use crate::handlers::test_handlers::test_handlers::*;
use actix_web::web;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure
        .service(home)
        .service(web::scope("/hello").service(index).service(hello));
}
