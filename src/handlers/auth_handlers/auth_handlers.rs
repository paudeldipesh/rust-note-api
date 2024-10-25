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
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct RegisterUserRequest {
    #[schema(example = "random", required = true)]
    pub username: String,
    #[schema(example = "random@gmail.com", required = true)]
    pub email: String,
    #[schema(example = "random", format = "password", required = true)]
    pub password: String,
}

#[utoipa::path(
    path = "/user/register",
    request_body = RegisterUserRequest,
    responses(
        (status = 200, description = "Successfully registered a new user."),
        (status = 500, description = "Failed to register the user due to an internal error."),
    )
)]
#[post("/register")]
pub async fn register_user(
    state: Data<AppState>,
    body: Json<RegisterUserRequest>,
) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    let hashed_password: String = match bcrypt::hash(&body.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "password hashing failed" }))
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
            .json(serde_json::json!({ "message": "failed to create user" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "unable to create user" })),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct LoginUserRequest {
    #[schema(example = "random@gmail.com", required = true)]
    pub email: String,
    #[schema(example = "random", format = "password", required = true)]
    pub password: String,
}

#[utoipa::path(
    path = "/user/login",
    request_body = LoginUserRequest,
    responses(
        (status = 200, description = "Successfully logged in. Returns a bearer token along with user information"),
        (status = 401, description = "Unauthorized: Invalid email or password."),
        (status = 500, description = "Internal Server Error: Unable to process the login request."),
    ),
    security(
        ("basic_auth" = [])
    )
)]
#[post("/login")]
pub async fn login_user(state: Data<AppState>, body: Json<LoginUserRequest>) -> impl Responder {
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
                            "user": {
                                "email": user.email,
                                "username": user.username,
                            }
                        }))
                    }
                    Err(_) => HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "message": "failed to generate token" })),
                }
            } else {
                HttpResponse::Unauthorized()
                    .json(serde_json::json!({ "message": "invalid credentials" }))
            }
        }
        Ok(Err(_)) => HttpResponse::Unauthorized()
            .json(serde_json::json!({ "message": "invalid email or password" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "unable to login user" })),
    }
}

#[utoipa::path(
    path = "/user/logout",
    responses(
        (status = 200, description = "Successfully logged out. The user's session has been terminated."),
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
        .json(serde_json::json!({ "message": "user logged out" }))
}
