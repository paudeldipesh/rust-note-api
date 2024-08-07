use actix_web::{get, web, Responder};

#[utoipa::path(
    path = "/",
    responses(
        (status = 200, description = "Display 'Welcome to Actix Web!' message"),
    ),
)]
#[get("/")]
pub async fn home() -> impl Responder {
    "Welcome to Actix Web!"
}

#[utoipa::path(
    path = "/test/hello-world",
    responses(
        (status = 200, description = "Display 'Hello, World!' message"),
    ),
)]
#[get("/hello-world")]
pub async fn index() -> impl Responder {
    "Hello, World!"
}

#[utoipa::path(
    path = "/test/{name}",
    responses(
        (status = 200, description = "Display 'Hello {name}!' message"),
    ),
)]
#[get("/{name}")]
pub async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}
