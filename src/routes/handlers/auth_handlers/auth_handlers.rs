use super::messages::*;
use crate::utils::{
    db::{AppState, DbActor},
    jwt::encode_jwt,
};
use actix::Addr;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct CreateUserBody {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

#[post("/register")]
pub async fn register_user(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(CreateUser {
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            username: body.username.clone(),
            email: body.email.clone(),
        })
        .await
    {
        Ok(Ok(user)) => HttpResponse::Ok().json(user),
        Ok(Err(_)) => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to create user" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to create user" })),
    }
}

#[derive(Deserialize)]
pub struct LoginUserBody {
    pub email: String,
    pub username: String,
}

#[derive(Serialize)]
pub struct LoginUserResponse {
    pub email: String,
    pub username: String,
}

#[post("/login")]
pub async fn login_user(state: Data<AppState>, body: Json<LoginUserBody>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(LoginUser {
            email: body.email.clone(),
            username: body.username.clone(),
        })
        .await
    {
        Ok(Ok(user)) => {
            let token_result = encode_jwt(user.email.clone(), user.id);
            match token_result {
                Ok(token) => HttpResponse::Ok().json(serde_json::json!({
                    "token": token,
                    "user": LoginUserResponse {
                        email: user.email,
                        username: user.username,
                    }
                })),
                Err(e) => HttpResponse::InternalServerError().json(
                    serde_json::json!({ "message": format!("Failed to generate token: {}", e) }),
                ),
            }
        }
        Ok(Err(_)) => HttpResponse::Unauthorized()
            .json(serde_json::json!({ "message": "Invalid email or username" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to login user" })),
    }
}
