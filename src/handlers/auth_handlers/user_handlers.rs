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
    path = "/admin/dashboard/users",
    responses(
        (status = 200, description = "Successfully retrieved a list of all users."),
        (status = 404, description = "No users found in the database."),
        (status = 500, description = "Internal Server Error: Unable to retrieve users due to a server issue."),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/users")]
pub async fn fetch_users(state: Data<AppState>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(FetchUser).await {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),

        Ok(Err(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({ "message": "no users found" }))
        }

        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "unable to retrieve users" })),
    }
}

#[utoipa::path(
    path = "/auth/user",
    responses(
        (status = 200, description = "Successfully retrieved the authenticated user's information."),
        (status = 401, description = "Unauthorized access: Invalid or missing authentication credentials."),
        (status = 500, description = "Internal Server Error: Unable to retrieve user information due to a server issue."),
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
                .json(serde_json::json!({"message": "unauthorized access"}));
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
            .json(serde_json::json!({ "message": "invalid email or password" })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "unable to retrieve user" })),
    }
}
