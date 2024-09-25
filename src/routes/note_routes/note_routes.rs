use crate::routes::{
    handlers::note_handlers::note_handlers::*, middlewares::auth_middlewares::check_auth_middleware,
};
use actix_web::web;
use actix_web_lab::middleware::from_fn;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure
        .service(web::scope("/api").service(fetch_all_notes))
        .service(
            web::scope("/secure/api")
                .wrap(from_fn(check_auth_middleware))
                .service(fetch_user_notes)
                .service(create_user_notes)
                .service(update_user_note)
                .service(delete_user_note),
        );
}
