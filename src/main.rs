use actix_web::{get, web, App, HttpServer, Responder};
mod utils;

#[get("/")]
async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let address: String = (*utils::constants::ADDRESS).clone();
    let port: u16 = (*utils::constants::PORT).clone();

    println!("Server running at http://{}:{}", address, port);

    HttpServer::new(|| App::new().service(index).service(hello))
        .bind((address, port))?
        .run()
        .await
}
