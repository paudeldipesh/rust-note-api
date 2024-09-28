use crate::utils::jwt::Claims;
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct UserInfo {
    moonpay_token: String,
}

#[utoipa::path(
    path = "/transaction/buy/info",
    params(
        ("moonpay_token" = String, Query, description = "Moonpay token of the user"),
    ),
    responses(
        (status = 200, description = "Handles buy info queries"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/buy/info")]
pub async fn buy_information(req: HttpRequest, query: web::Query<UserInfo>) -> impl Responder {
    let UserInfo { moonpay_token } = query.into_inner();

    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Unauthorized access" }));
        }
    };

    let url: String = format!(
        "https://api.moonpay.com/v1/transactions?externalCustomerId={}",
        claims.id
    );

    let client: Client = Client::new();

    match client.get(url).bearer_auth(moonpay_token).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => HttpResponse::Ok().json(json),
                    Err(_) => HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to parse response" })),
                }
            } else {
                let status_code: reqwest::StatusCode = response.status();

                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to fetch transaction info",
                    "status": status_code.as_u16(),
                }))
            }
        }
        Err(error) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Request error",
            "details": error.to_string(),
        })),
    }
}
