use actix_web::{App, HttpServer};
mod models;
mod routes;
mod schema;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address: String = (*utils::constants::ADDRESS).clone();
    let port: u16 = (*utils::constants::PORT).clone();

    println!("Server running at http://{}:{}", address, port);

    HttpServer::new(|| App::new().configure(routes::test_routes::configuration))
        .bind((address, port))?
        .run()
        .await
}
