use actix::{Addr, SyncArbiter};
use actix_web::{web::Data, App, HttpServer};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use utils::db::{get_pool, AppState, DbActor};
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address: String = (*utils::constants::ADDRESS).clone();
    let port: u16 = (*utils::constants::PORT).clone();

    let db_url: String = (*utils::constants::DATABASE_URL).clone();
    let pool: Pool<ConnectionManager<PgConnection>> = get_pool(&db_url);
    let db_addr: Addr<DbActor> = SyncArbiter::start(5, move || DbActor(pool.clone()));

    println!("Server running at http://{}:{}", address, port);

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState {
                db: db_addr.clone(),
            }))
            .configure(routes::test_routes::test_routes::configuration)
            .configure(routes::note_routes::note_routes::configuration)
    })
    .bind((address, port))?
    .run()
    .await
}
