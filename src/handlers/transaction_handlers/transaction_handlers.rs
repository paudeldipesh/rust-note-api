use crate::utils::{self, jwt::Claims};
use actix_web::{get, web, HttpMessage, HttpRequest, HttpResponse, Responder};
use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct BuyListsQuery {
    moonpay_token: String,
}

#[utoipa::path(
    path = "/secure/transaction/buy/lists",
    params(
        ("moonpay_token" = String, Query, description = "Moonpay token of the user"),
    ),
    responses(
        (status = 200, description = "Handles buy lists queries"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/buy/lists")]
pub async fn get_buy_lists(req: HttpRequest, query: web::Query<BuyListsQuery>) -> impl Responder {
    let BuyListsQuery { moonpay_token } = query.into_inner();

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

#[derive(Deserialize)]
struct BuyQuoteInfo {
    crypto_code: String,
    fiat_code: String,
    crypto_amount: u32,
}

#[utoipa::path(
    path = "/transaction/buy/quote",
    params(
        ("crypto_code" = String, Query, description = "Crypto Currency Code"),
        ("fiat_code" = String, Query, description = "Base Currency Code"),
        ("crypto_amount" = u32, Query, description = "Quote Currency Amount"),
    ),
    responses(
        (status = 200, description = "Handles buy quote queries"),
    )
)]
#[get("/buy/quote")]
pub async fn get_buy_quote(query: web::Query<BuyQuoteInfo>) -> impl Responder {
    let BuyQuoteInfo {
        crypto_code,
        fiat_code,
        crypto_amount,
    } = query.into_inner();

    let api_key: String = (*utils::constants::MOONPAY_API_KEY).clone();

    let url: String = format!(
        "https://api.moonpay.com/v3/currencies/{}/buy_quote?quoteCurrencyAmount={}&baseCurrencyCode={}&apiKey={}",
        crypto_code, crypto_amount, fiat_code, api_key
    );

    let client: Client = Client::new();

    match client.get(url).send().await {
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
                    "error": "Failed to fetch buy quote info",
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

#[derive(Deserialize)]
struct BuyInfoQuery {
    transaction_id: String,
}

#[utoipa::path(
    path = "/transaction/buy/info",
    params(
        ("transaction_id" = String, Query, description = "Transaction id of crypto")
    ),
    responses(
        (status = 200, description = "Response the buy transaction of crypto"),
    )
)]
#[get("/buy/info")]
pub async fn get_buy_information(query: web::Query<BuyInfoQuery>) -> impl Responder {
    let BuyInfoQuery { transaction_id } = query.into_inner();

    let api_key: String = (*utils::constants::MOONPAY_API_KEY).clone();

    let url: String = format!(
        "https://api.moonpay.com/v1/transactions/{}?apiKey={}",
        transaction_id, api_key
    );

    let client: Client = Client::new();

    match client.get(url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<serde_json::Value>().await {
                    Ok(json) => HttpResponse::Ok().json(json),
                    Err(_) => HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "error": "Failed to parse response" })),
                }
            } else {
                let status_code: reqwest::StatusCode = response.status();
                let error_body = match response.text().await {
                    Ok(body) => body,
                    Err(_) => "Failed to read error response body".to_string(),
                };

                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to fetch buy info",
                    "status": status_code.as_u16(),
                    "details": error_body,
                }))
            }
        }
        Err(error) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Request error",
            "details": error.to_string(),
        })),
    }
}

#[derive(Deserialize)]
struct SwapInfo {
    moonpay_token: String,
    transaction_id: String,
}

#[utoipa::path(
    path = "/transaction/swap/info",
    params(
        ("moonpay_token" = String, Query, description = "Auth token - moonpay"),
        ("transaction_id" = String, Query, description = "Transaction id of swap"),
    ),
    responses(
        (status = 200, description = "Handles buy quote queries"),
    )
)]
#[get("/swap/info")]
pub async fn get_swap_transaction(query: web::Query<SwapInfo>) -> impl Responder {
    let SwapInfo {
        moonpay_token,
        transaction_id,
    } = query.into_inner();

    let api_key: String = (*utils::constants::MOONPAY_API_KEY).clone();

    let url: String = format!(
        "https://api.moonpay.com/v4/swap/transaction/{}?apiKey={}",
        transaction_id, api_key
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
                let error_body = match response.text().await {
                    Ok(body) => body,
                    Err(_) => "Failed to read error response body".to_string(),
                };

                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to fetch buy quote info",
                    "status": status_code.as_u16(),
                    "details": error_body,
                }))
            }
        }
        Err(error) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Request error",
            "details": error.to_string(),
        })),
    }
}
