use super::messages::*;
use crate::utils::jwt::Claims;
#[allow(dead_code)]
use crate::utils::{
    db::{AppState, DbActor},
    jwt::encode_jwt,
};
use actix::Addr;
use actix_web::{
    post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct CreateUserBody {
    #[schema(example = "testuser", required = true)]
    pub username: String,
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
    #[schema(example = "testuser", required = true)]
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

#[derive(Deserialize, ToSchema)]
pub struct LoginUserBody {
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
    #[schema(example = "testuser", required = true)]
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
        (status = 200, description = "Login using credentials, returns bearer token", body = LoginUser),
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

#[derive(Deserialize)]
pub struct LogoutUserBody {
    pub email: String,
}
#[post("/logout")]
pub async fn logout_user(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<LogoutUserBody>,
) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

    if body.email != claims.email {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"message": "Email does not match"}));
    }

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(LogoutUser {
            email: claims.email,
        })
        .await
    {
        Ok(Ok(_)) => HttpResponse::Ok()
            .json(serde_json::json!({ "status": "success", "message": "Successfully logged out" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "status": "fail", "message": "Unable to logout user" })),
    }
}

#[derive(Deserialize)]
pub struct GenerateOTPBody {
    pub email: String,
}

#[post("/otp/generate")]
pub async fn generate_otp_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<GenerateOTPBody>,
) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

    if body.email != claims.email {
        return HttpResponse::Forbidden()
            .json(serde_json::json!({"message": "Email does not match."}));
    }

    let db: Addr<DbActor> = state.as_ref().db.clone();

    let mut rng: ThreadRng = rand::thread_rng();
    let data_byte: [u8; 21] = rng.gen();
    let base32_string: String =
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &data_byte);

    let totp: TOTP = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(base32_string).to_bytes().unwrap(),
    )
    .unwrap();

    let otp_base32: String = totp.get_secret_base32();
    let email: String = claims.email;
    let issuer: &str = "NoteAPI";
    let otp_auth_url: String =
        format!("otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}");

    let opt_verified: bool = false;
    let opt_enabled: bool = true;

    match db
        .send(GenerateOTPMessage {
            email,
            opt_verified,
            opt_enabled,
            opt_auth_url: otp_auth_url.clone(),
            opt_base32: otp_base32.clone(),
        })
        .await
    {
        Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "otp_auth_url": otp_auth_url,
            "otp_base32": otp_base32,
        })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to generate OTP" })),
    }
}

#[derive(Deserialize)]
pub struct VerifyOTPBody {
    pub otp_token: String,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
#[post("/otp/verify")]
pub async fn verify_otp_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<VerifyOTPBody>,
) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized().json(GenericResponse {
                status: String::from("fail"),
                message: String::from("unauthorized access"),
            });
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();
    let user_email: String = claims.email.clone();

    match db
        .send(LoginUser {
            email: user_email.clone(),
            _password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => {
            let otp_base32: String = user.opt_base32.clone().unwrap();

            let totp: TOTP = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                Secret::Encoded(otp_base32).to_bytes().unwrap(),
            )
            .unwrap();

            let is_valid: bool = totp.check_current(&body.otp_token).unwrap();

            if !is_valid {
                let json_error: GenericResponse = GenericResponse {
                    status: String::from("fail"),
                    message: String::from("invalid token"),
                };
                return HttpResponse::Forbidden().json(json_error);
            }

            match db
                .send(GenerateOTPMessage {
                    email: user_email,
                    opt_verified: true,
                    opt_enabled: true,
                    opt_base32: user.opt_base32.clone().unwrap(),
                    opt_auth_url: user.opt_auth_url.clone().unwrap(),
                })
                .await
            {
                Ok(Ok(updated_user)) => HttpResponse::Ok().json(serde_json::json!({
                    "otp_verified": true,
                    "user": updated_user,
                })),
                _ => HttpResponse::InternalServerError().json(GenericResponse {
                    status: String::from("fail"),
                    message: String::from("failed to update OTP status"),
                }),
            }
        }
        Ok(Err(_)) => HttpResponse::InternalServerError().json(GenericResponse {
            status: String::from("fail"),
            message: String::from("failed to retrieve user"),
        }),
        _ => HttpResponse::InternalServerError().json(GenericResponse {
            status: String::from("fail"),
            message: String::from("internal server error"),
        }),
    }
}
