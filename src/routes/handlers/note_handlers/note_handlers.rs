use super::messages::{CreateNote, FetchUser, FetchUserNotes};
use crate::utils::db::{AppState, DbActor};
use actix::Addr;
use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde::Deserialize;

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateNoteBody {
    pub title: String,
    pub content: String,
}

#[get("/users")]
pub async fn fetch_users(app_state: Data<AppState>) -> impl Responder {
    let db: Addr<DbActor> = app_state.as_ref().db.clone();

    match db.send(FetchUser).await {
        Ok(Ok(users)) => HttpResponse::Ok().json(users),
        Ok(Err(_)) => HttpResponse::NotFound().json("No users found"),
        _ => HttpResponse::InternalServerError().json("Unable to retrieve users"),
    }
}

#[get("/user/{id}/notes")]
pub async fn fetch_user_notes(app_state: Data<AppState>, path: Path<i32>) -> impl Responder {
    let id: i32 = path.into_inner();

    let db: Addr<DbActor> = app_state.as_ref().db.clone();

    match db.send(FetchUserNotes { user_id: id }).await {
        Ok(Ok(notes)) => HttpResponse::Ok().json(notes),
        Ok(Err(_)) => HttpResponse::NotFound().json(format!("No notes for user {id}")),
        _ => HttpResponse::InternalServerError().json("Unable to retrieve user notes"),
    }
}

#[post("/user/{id}/note")]
pub async fn create_user_notes(
    state: Data<AppState>,
    path: Path<i32>,
    body: Json<CreateNoteBody>,
) -> impl Responder {
    let id: i32 = path.into_inner();

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(CreateNote {
            title: body.title.to_string(),
            content: body.content.to_string(),
            created_by: id,
        })
        .await
    {
        Ok(Ok(info)) => HttpResponse::Ok().json(info),
        _ => HttpResponse::InternalServerError().json("Failed to create article"),
    }
}
