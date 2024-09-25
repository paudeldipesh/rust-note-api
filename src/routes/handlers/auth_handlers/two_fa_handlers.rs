use actix::Addr;
use actix_web::{
    get, post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};
use utoipa::ToSchema;

use crate::{
    utils::{
        db::{AppState, DbActor},
        jwt::Claims,
    },
    LoginAndGetUser, OTPMessage,
};

#[utoipa::path(
    path = "/auth/otp/generate",
    responses(
        (status = 200, description = "OTP generated"),
        (status = 500, description = "failed to generate OTP"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/otp/generate")]
pub async fn generate_otp_handler(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

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
        .send(OTPMessage {
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
                .send(OTPMessage {
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

    let user_email: String = claims.email.clone();
    let otp_token: String = body.otp_token.clone();

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
                    message: String::from("OTP not validated"),
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

#[utoipa::path(
    path = "/auth/otp/disable",
    responses(
        (status = 200, description = "OTP disabled"),
        (status = 500, description = "failed to disable OTP"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/otp/disable")]
pub async fn disable_otp_handler(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(OTPMessage {
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
