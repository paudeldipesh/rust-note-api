use crate::{
    utils::{
        db::{AppState, DbActor},
        jwt::Claims,
    },
    FetchUser, LoginAndGetUser,
};
use actix::Addr;
use actix_web::{get, web::Data, HttpMessage, HttpRequest, HttpResponse, Responder};

#[utoipa::path(
    path = "/auth/users",
    responses(
        (status = 200, description = "Get all users"),
        (status = 404, description = "No users found"),
        (status = 500, description = "Unable to retrieve users"),
    ),
)]
#[get("/users")]
pub async fn fetch_users(state: Data<AppState>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(FetchUser).await {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),

        Ok(Err(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({ "message": "No users found" }))
        }

        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve users" })),
    }
}

#[utoipa::path(
    path = "/auth/user",
    responses(
        (status = 200, description = "Retrieve the user"),
        (status = 401, description = "Unauthorized access"),
        (status = 500, description = "Server error"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/user")]
pub async fn get_user(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(LoginAndGetUser {
            email: claims.email.clone(),
            password: String::new(),
        })
        .await
    {
        Ok(Ok(user)) => HttpResponse::Ok().json(serde_json::json!({
            "user": user
        })),
        Ok(Err(_)) => HttpResponse::Unauthorized()
            .json(serde_json::json!({ "message": "Invalid email or password" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve user" })),
    }
}
