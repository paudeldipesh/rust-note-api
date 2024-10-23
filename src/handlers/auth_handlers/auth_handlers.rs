use super::messages::*;
use crate::utils::{
    db::{AppState, DbActor},
    jwt::encode_jwt,
};
use actix::Addr;
use actix_web::{
    cookie::{
        time::{Duration, OffsetDateTime},
        Cookie,
    },
    get, post,
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

#[utoipa::path(
    path = "/user/register",
    request_body = CreateUserBody,
    responses(
        (status = 200, description = "Create a new user", body = CreateUser),
        (status = 500, description = "Unable to create user"),
    )
)]
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

#[utoipa::path(
    path = "/user/login",
    request_body = LoginUserBody,
    responses(
        (status = 200, description = "Login using credentials, returns bearer token", body = LoginAndGetUser),
        (status = 401, description = "Basic auth required"),
        (status = 500, description = "Internal server error"),
    ),
    security(
        ("basic_auth" = [])
    )
)]
#[post("/login")]
pub async fn login_user(state: Data<AppState>, body: Json<LoginUserBody>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(LoginAndGetUser {
            email: body.email.clone(),
            password: body.password.clone(),
        })
        .await
    {
        Ok(Ok(user)) => {
            let is_valid = bcrypt::verify(&body.password, &user.password);
            if is_valid.unwrap_or(false) {
                let token_result = encode_jwt(user.email.clone(), user.id, user.role);
                match token_result {
                    Ok(token) => {
                        let oneday: OffsetDateTime = OffsetDateTime::now_utc() + Duration::days(1);

                        let cookie = Cookie::build("token", token.clone())
                        .path("/")
                        .http_only(true)
                        .secure(false)
                        .expires(oneday)
                        .finish();

                        HttpResponse::Ok().cookie(cookie).json(serde_json::json!({
                        "token": token,
                        "user": LoginUserResponse {
                            email: user.email,
                            username: user.username,
                        }
                    }))},
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

#[utoipa::path(
    path = "/user/logout",
    responses(
        (status = 200, description = "User logout"),
        (status = 500, description = "Unable to logout user"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/logout")]
pub async fn logout_user() -> impl Responder {
    let now: OffsetDateTime = OffsetDateTime::now_utc();

    let cookie = Cookie::build("token", "logout")
        .path("/")
        .http_only(true)
        .secure(false)
        .expires(now)
        .finish();

    HttpResponse::Ok()
        .cookie(cookie)
        .json(serde_json::json!({ "status": "success", "message": "Successfully logged out" }))
}
