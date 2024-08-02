use actix_web::{get, web, Responder};

#[get("/")]
pub async fn home() -> impl Responder {
    "Welcome to Actix Web!"
}

#[get("/hello-world")]
pub async fn index() -> impl Responder {
    "Hello, World!"
}

#[get("/{name}")]
pub async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}
