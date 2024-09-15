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

#[derive(Deserialize, ToSchema)]
pub struct GetUserBody {
    #[schema(example = "testuser@gmail.com")]
    pub email: String,
    #[schema(example = "password123")]
    pub password: String,
}

#[utoipa::path(
    path = "/auth/user",
    request_body = GetUserBody,
    responses(
        (status = 200, description = "Retrive the user"),
        (status = 500, description = "Invalid credentials"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/user")]
pub async fn get_user(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<GetUserBody>,
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

    match db
        .send(LoginAndGetUser {
            email: claims.email.clone(),
            _password: body.password.clone(),
        })
        .await
    {
        Ok(Ok(user)) => match bcrypt::verify(&body.password, &user.password) {
            Ok(is_valid) if is_valid => HttpResponse::Ok().json(serde_json::json!({
                "user": user
            })),
            Ok(_) => HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Invalid email or password" })),
            Err(_) => HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Password verification failed" })),
        },

        Ok(Err(_)) => HttpResponse::Unauthorized()
            .json(serde_json::json!({ "message": "Invalid email or password" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve user" })),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct LogoutUserBody {
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
}
#[utoipa::path(
    path = "/auth/logout",
    request_body = LogoutUserBody,
    responses(
        (status = 200, description = "User logout"),
        (status = 500, description = "Unable to logout user"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

#[derive(Deserialize, ToSchema)]
pub struct GenerateOTPBody {
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
}
#[utoipa::path(
    path = "/auth/otp/generate",
    request_body = GenerateOTPBody,
    responses(
        (status = 200, description = "User logout"),
        (status = 500, description = "Unable to logout user"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
        .send(GenerateAndDisableOTPMessage {
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

#[derive(Deserialize, ToSchema)]
pub struct VerifyOTPBody {
    #[schema(example = "123456", required = true)]
    pub otp_token: String,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
#[utoipa::path(
    path = "/auth/otp/verify",
    request_body = VerifyOTPBody,
    responses(
        (status = 200, description = "OTP verified"),
        (status = 500, description = "failed to verify OTP"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
        .send(LoginAndGetUser {
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

            let opt_verified: bool = true;
            let opt_enabled: bool = true;

            match db
                .send(GenerateAndDisableOTPMessage {
                    email: user_email,
                    opt_verified,
                    opt_enabled,
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

#[derive(Deserialize, ToSchema)]
pub struct ValidateOTPBody {
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
    #[schema(example = "123456", required = true)]
    pub otp_token: String,
}
#[utoipa::path(
    path = "/auth/otp/validate",
    request_body = ValidateOTPBody,
    responses(
        (status = 200, description = "OTP validated"),
        (status = 500, description = "failed to validate OTP"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/otp/validate")]
pub async fn token_validate_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<ValidateOTPBody>,
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

    let user_email: String = body.email.clone();
    let otp_token: String = body.otp_token.clone();

    if user_email != claims.email {
        return HttpResponse::Forbidden().json(GenericResponse {
            status: String::from("fail"),
            message: String::from("email mismatch"),
        });
    }

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(LoginAndGetUser {
            email: user_email.clone(),
            _password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => {
            if !user.opt_verified.unwrap_or(false) {
                return HttpResponse::Forbidden().json(GenericResponse {
                    status: String::from("fail"),
                    message: String::from("OTP not verified"),
                });
            }

            let otp_base32: String = user.opt_base32.clone().unwrap();
            let totp: TOTP = TOTP::new(
                Algorithm::SHA1,
                6,
                1,
                30,
                Secret::Encoded(otp_base32).to_bytes().unwrap(),
            )
            .unwrap();

            let is_valid: bool = totp.check_current(&otp_token).unwrap();

            if !is_valid {
                return HttpResponse::Forbidden().json(GenericResponse {
                    status: String::from("fail"),
                    message: String::from("invalid OTP token"),
                });
            }

            HttpResponse::Ok().json(serde_json::json!({ "status": "success", "user": user }))
        }
        Ok(Err(_)) => HttpResponse::InternalServerError().json(GenericResponse {
            status: String::from("fail"),
            message: String::from("failed to retrieve user"),
        }),
        Err(_) => HttpResponse::InternalServerError().json(GenericResponse {
            status: String::from("fail"),
            message: String::from("internal server error"),
        }),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct DisableOTPBody {
    #[schema(example = "testuser@gmail.com", required = true)]
    pub email: String,
}
#[utoipa::path(
    path = "/auth/otp/disable",
    request_body = DisableOTPBody,
    responses(
        (status = 200, description = "OTP disabled"),
        (status = 500, description = "failed to disable OTP"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/otp/disable")]
pub async fn disable_otp_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<DisableOTPBody>,
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

    match db
        .send(GenerateAndDisableOTPMessage {
            email: claims.email,
            opt_verified: false,
            opt_enabled: false,
            opt_auth_url: String::new(),
            opt_base32: String::new(),
        })
        .await
    {
        Ok(Ok(data)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": data,
        })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to disable OTP" })),
    }
}
