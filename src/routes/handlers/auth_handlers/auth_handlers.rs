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
    pub username: String,
    pub email: String,
    pub password: String,
}

#[post("/register")]
pub async fn register_user(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    let hashed_password: String = match bcrypt::hash(&body.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Password hashing failed" }))
        }
    };

    match db
        .send(CreateUser {
            username: body.username.clone(),
            email: body.email.clone(),
            password: hashed_password,
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
    pub password: String,
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
            _password: body.password.clone(),
        })
        .await
    {
        Ok(Ok(user)) => {
            let is_valid = bcrypt::verify(&body.password, &user.password);
            if is_valid.unwrap_or(false) {
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
            } else {
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "message": "Invalid email or password" }))
            }
        }
        Ok(Err(_)) => HttpResponse::Unauthorized()
            .json(serde_json::json!({ "message": "Invalid email or password" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to login user" })),
    }
}
