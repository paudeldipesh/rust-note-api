use crate::{
    utils::{
        db::{AppState, DbActor},
        jwt::Claims,
    },
    LoginAndGetUser,
};
use actix::Addr;
use actix_web::{
    post,
    web::{Data, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use serde::Deserialize;
use utoipa::ToSchema;

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
