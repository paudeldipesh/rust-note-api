use actix_web::{get, web::Path, HttpResponse, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct MessageResponse {
    message: String,
}

#[utoipa::path(
    path = "/",
    responses(
        (status = 200, description = "JSON welcome message response"),
    ),
)]
#[get("/")]
pub async fn home() -> impl Responder {
    let response: MessageResponse = MessageResponse {
        message: String::from("welcome to rust not api"),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    path = "/hello/hello-world",
    responses(
        (status = 200, description = "JSON hello world message response"),
    ),
)]
#[get("/hello-world")]
pub async fn index() -> impl Responder {
    let response: MessageResponse = MessageResponse {
        message: String::from("hello world!"),
    };

    HttpResponse::Ok().json(response)
}

#[utoipa::path(
    path = "/hello/{name}",
    responses(
        (status = 200, description = "JSON hello user message response"),
    ),
)]
#[get("/{name}")]
pub async fn hello(name: Path<String>) -> impl Responder {
    let incomming_name: String = name.into_inner();

    let response: MessageResponse = MessageResponse {
        message: format!("Hello, {}", incomming_name),
    };

    HttpResponse::Ok().json(response)
}
