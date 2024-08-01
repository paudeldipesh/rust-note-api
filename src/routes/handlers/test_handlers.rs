use actix_web::{get, web, Responder};

#[get("/hello-world")]
pub async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
pub async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}
