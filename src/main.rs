use actix::{Addr, SyncArbiter};
use actix_web::{web::Data, App, HttpServer};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use utils::{
    db::{get_pool, AppState, DbActor},
    jwt::Claims,
};
mod handlers;
mod middlewares;
mod models;
mod routes;
use actix_governor::{Governor, GovernorConfigBuilder};
use handlers::{
    auth_handlers::{auth_handlers::*, messages::*, two_fa_handlers::*, user_handlers::*},
    note_handlers::note_handlers::*,
    test_handlers::test_handlers::*,
    transaction_handlers::transaction_handlers::*,
};
use routes::{
    admin_routes::admin_routes, auth_routes::auth_routes, note_routes::note_routes,
    test_routes::test_routes, transaction_routes::transaction_routes,
};
mod schema;
mod utils;
use utoipa::{
    openapi::{
        security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
        Components, OpenApi as OpenApiType,
    },
    Modify, OpenApi as OpenApiDerive,
};
use utoipa_swagger_ui::SwaggerUi;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address: String = (*utils::constants::ADDRESS).clone();
    let port: u16 = (*utils::constants::PORT).clone();

    let db_url: String = (*utils::constants::DATABASE_URL).clone();
    let pool: Pool<ConnectionManager<PgConnection>> = get_pool(&db_url);
    let db_addr: Addr<DbActor> = SyncArbiter::start(5, move || DbActor(pool.clone()));

    println!("Server running at http://{}:{}", address, port);

    #[derive(OpenApiDerive)]
    #[openapi(
    paths(
        login_user,
        register_user,
        hello,
        home,
        index,
        fetch_users,
        fetch_notes,
        create_user_notes,
        fetch_user_notes,
        update_user_note,
        delete_user_note,
        generate_otp_handler,
        verify_otp_handler,
        logout_user,
        token_validate_handler,
        disable_otp_handler,
        get_user,
        get_buy_lists,
        get_buy_quote,
        get_buy_information,
        get_swap_transaction,
        update_password,
        delete_user,
    ),
    components(
        schemas(
            Claims,
            RegisterUserRequest,
            LoginUserRequest,
            CreateNoteRequest,
            UpdateNoteRequest,
            VerifyOTPRequest,
            ValidateOTPRequest,
            UpdatePasswordRequest,
        )
    ),
    modifiers(&SecurityAddon)
    )]
    struct ApiDoc;

    struct SecurityAddon;
    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components: &mut Components = openapi.components.as_mut().unwrap();

            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );

            components.add_security_scheme(
                "basic_auth",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Basic).build()),
            )
        }
    }

    let open_api: OpenApiType = ApiDoc::openapi();

    let governor_conf = GovernorConfigBuilder::default()
        .requests_per_minute(60)
        .burst_size(30)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                db: db_addr.clone(),
            }))
            .wrap(Governor::new(&governor_conf))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", open_api.clone()),
            )
            .configure(test_routes::configuration)
            .configure(note_routes::configuration)
            .configure(auth_routes::configuration)
            .configure(transaction_routes::configuration)
            .configure(admin_routes::configuration)
    })
    .bind((address, port))?
    .run()
    .await
}
