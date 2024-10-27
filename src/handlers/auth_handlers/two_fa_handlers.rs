use crate::{
    utils::{
        db::{AppState, DbActor},
        jwt::Claims,
    },
    LoginAndGetUser, OTPMessage,
};
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

#[utoipa::path(
    path = "/auth/otp/generate",
    responses(
        (status = 200, description = "OTP successfully generated."),
        (status = 500, description = "Failed to generate the OTP."),
        (status = 401, description = "Unauthorized access if the JWT token is missing or invalid."),
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
                .json(serde_json::json!({"message": "unauthorized access"}));
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
    let issuer: &str = "RustNoteAPI";
    let otp_auth_url: String =
        format!("otpauth://totp/{issuer}:{email}?secret={otp_base32}&issuer={issuer}");

    let otp_verified: bool = false;
    let otp_enabled: bool = true;

    match db
        .send(OTPMessage {
            email,
            otp_verified,
            otp_enabled,
            otp_auth_url: Some(otp_auth_url.clone()),
            otp_base32: Some(otp_base32.clone()),
        })
        .await
    {
        Ok(Ok(_)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "otp_auth_url": otp_auth_url,
            "otp_base32": otp_base32,
        })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "failed to generate otp" })),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct VerifyOTPRequest {
    #[schema(example = "123456")]
    pub otp_token: String,
}

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}
#[utoipa::path(
    path = "/auth/otp/verify",
    request_body = VerifyOTPRequest,
    responses(
        (status = 200, description = "OTP successfully verified, returns updated user details with OTP verification status"),
        (status = 403, description = "Invalid OTP token."),
        (status = 500, description = "Failed to verify OTP or update OTP status."),
        (status = 401, description = "Unauthorized access, JWT token is missing or invalid."),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/otp/verify")]
pub async fn verify_otp_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<VerifyOTPRequest>,
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
            password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => {
            let otp_base32: String = user.otp_base32.clone().unwrap();

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

            let otp_verified: bool = true;
            let otp_enabled: bool = true;

            match db
                .send(OTPMessage {
                    email: user_email,
                    otp_verified,
                    otp_enabled,
                    otp_base32: Some(user.otp_base32.unwrap().clone()),
                    otp_auth_url: Some(user.otp_auth_url.unwrap().clone()),
                })
                .await
            {
                Ok(Ok(updated_user)) => HttpResponse::Ok().json(serde_json::json!({
                    "otp_verified": true,
                    "user": updated_user,
                })),
                _ => HttpResponse::InternalServerError().json(GenericResponse {
                    status: String::from("fail"),
                    message: String::from("failed to update otp status"),
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
pub struct ValidateOTPRequest {
    #[schema(example = "123456")]
    pub otp_token: String,
}
#[utoipa::path(
    path = "/auth/otp/validate",
    request_body = ValidateOTPRequest,
    responses(
        (status = 200, description = "OTP successfully validated, returns the user details if OTP is valid."),
        (status = 403, description = "OTP not validated or invalid OTP token."),
        (status = 500, description = "Failed to validate OTP or retrieve user."),
        (status = 401, description = "Unauthorized access, JWT token is missing or invalid."),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[post("/otp/validate")]
pub async fn token_validate_handler(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<ValidateOTPRequest>,
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
            password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => {
            if !user.otp_verified.unwrap_or(false) {
                return HttpResponse::Forbidden().json(GenericResponse {
                    status: String::from("fail"),
                    message: String::from("otp not validated"),
                });
            }

            let otp_base32: String = user.otp_base32.clone().unwrap();
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
                    message: String::from("invalid otp token"),
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
        (status = 200, description = "OTP successfully disabled, returns confirmation."),
        (status = 500, description = "Failed to disable OTP."),
        (status = 401, description = "Unauthorized access, JWT token is missing or invalid."),
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
                .json(serde_json::json!({"message": "unauthorized access"}));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(OTPMessage {
            email: claims.email,
            otp_verified: false,
            otp_enabled: false,
            otp_auth_url: Some(String::new()),
            otp_base32: Some(String::new()),
        })
        .await
    {
        Ok(Ok(data)) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "data": data,
        })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "failed to disable otp" })),
    }
}
