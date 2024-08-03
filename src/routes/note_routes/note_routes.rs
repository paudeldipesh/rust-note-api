use crate::routes::handlers::note_handlers::note_handlers::*;
use actix_web::web;

pub fn configuration(configure: &mut web::ServiceConfig) {
    configure.service(
        web::scope("/api")
            .service(fetch_users)
            .service(fetch_user_notes)
            .service(create_user_notes)
            .service(update_user_note)
            .service(delete_user_notes)
            .service(fetch_all_notes),
    );
}
