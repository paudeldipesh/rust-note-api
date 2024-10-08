use crate::utils::jwt::{decode_jwt, Claims};
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    http::header::{HeaderValue, AUTHORIZATION},
    Error, HttpMessage,
};
use actix_web_lab::middleware::Next;
use jsonwebtoken::{errors::ErrorKind, TokenData};

pub async fn check_auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if !req.cookie("token").is_some() {
        return Err(ErrorUnauthorized(
            serde_json::json!({ "message": "Token is not available in the cookie" }),
        ));
    }

    let auth: Option<&HeaderValue> = req.headers().get(AUTHORIZATION);

    if auth.is_none() {
        return Err(ErrorUnauthorized(
            serde_json::json!({ "message": "Provide an authentication token in the request" }),
        ));
    };

    let token: String = auth
        .unwrap()
        .to_str()
        .unwrap()
        .replace("Bearer ", "")
        .to_owned();

    let claim: TokenData<Claims> = match decode_jwt(token) {
        Ok(claim_data) => claim_data,
        Err(e) => match e.kind() {
            ErrorKind::ExpiredSignature => {
                return Err(ErrorUnauthorized(
                    serde_json::json!({ "message": "Token has expired" }),
                ));
            }
            ErrorKind::InvalidSignature => {
                return Err(ErrorUnauthorized(
                    serde_json::json!({ "message": "Invalid token signature" }),
                ));
            }
            _ => {
                return Err(ErrorUnauthorized(
                    serde_json::json!({ "message": "Invalid token", "details": e.to_string() }),
                ));
            }
        },
    };

    req.extensions_mut().insert(claim.claims);

    next.call(req).await.map_err(|err| {
        ErrorInternalServerError(
            serde_json::json!({ "message": "Internal server error", "details": err.to_string() })
                .to_string(),
        )
    })
}
