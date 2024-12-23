use super::messages::*;
use crate::utils::{
    db::{AppState, DbActor},
    jwt::{encode_jwt, Claims},
};
use actix::Addr;
use actix_web::{
    cookie::{
        time::{Duration, OffsetDateTime},
        Cookie,
    },
    delete, get, post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
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

#[derive(Deserialize, ToSchema)]
pub struct UpdatePasswordRequest {
    #[schema(example = "random", required = true)]
    old_password: String,
    #[schema(example = "random", required = true)]
    new_password: String,
}
#[utoipa::path(
    path = "/auth/update-password",
    request_body(
        content = UpdatePasswordRequest,
        description = "Request body containing old and new passwords.",
    ),
    responses(
        (status = 200, description = "Password updated successfully."),
        (status = 400, description = "Invalid old password."),
        (status = 401, description = "Unauthorized access."),
        (status = 500, description = "Failed to update the password due to an internal error."),
    ),
    security(
        ("basic_auth" = [])
    )
)]
#[post("/update-password")]
pub async fn update_password(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<UpdatePasswordRequest>,
) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "unauthorized access"}));
        }
    };

    match db
        .send(LoginAndGetUser {
            email: claims.email.clone(),
            password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => {
            if bcrypt::verify(&body.old_password, &user.password).unwrap_or(false) {
                let hashed_password = match bcrypt::hash(&body.new_password, bcrypt::DEFAULT_COST) {
                    Ok(hash) => hash,
                    Err(_) => {
                        return HttpResponse::InternalServerError().json(serde_json::json!({
                            "message": "password hashing failed"
                        }));
                    }
                };

                match db
                    .send(UpdateUserPassword {
                        user_id: claims.id,
                        new_password: hashed_password,
                    })
                    .await
                {
                    Ok(Ok(_)) => {
                        HttpResponse::Ok().json(serde_json::json!({"message": "password updated"}))
                    }
                    _ => HttpResponse::InternalServerError()
                        .json(serde_json::json!({"message": "failed to update password"})),
                }
            } else {
                HttpResponse::BadRequest()
                    .json(serde_json::json!({"message": "invalid old password"}))
            }
        }
        Ok(Err(_)) => HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "failed to retrieve user"})),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({"message": "internal server error"})),
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

#[utoipa::path(
    path = "/auth/delete",
    responses(
        (status = 200, description = "Successfully deleted the user account."),
        (status = 401, description = "Unauthorized: User is not logged in or token is invalid."),
        (status = 500, description = "Failed to delete the user account due to an internal error."),
    ),
    security(
        ("basic_auth" = [])
    )
)]
#[delete("/delete")]
pub async fn delete_user(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "unauthorized access"}));
        }
    };

    let now: OffsetDateTime = OffsetDateTime::now_utc();

    let cookie = Cookie::build("token", "delete")
        .path("/")
        .http_only(true)
        .secure(false)
        .expires(now)
        .finish();

    let result = db.send(DeleteUser { user_id: claims.id }).await;

    match result {
        Ok(Ok(rows_deleted)) if rows_deleted > 0 => HttpResponse::Ok()
            .cookie(cookie)
            .json(serde_json::json!({ "message": "user deleted" })),

        Ok(Ok(_)) => HttpResponse::NotFound().json(serde_json::json!({
            "message": "user not found"
        })),

        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "failed to delete user" })),
    }
}
