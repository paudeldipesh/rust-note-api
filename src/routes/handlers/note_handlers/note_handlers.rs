use super::messages::*;
use crate::utils::db::{AppState, DbActor};
use actix::Addr;
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserBody {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct CreateNoteBody {
    pub title: String,
    pub content: String,
    pub created_on: Option<DateTime<Utc>>,
}

#[derive(Deserialize)]
pub struct UpdateNoteBody {
    pub title: Option<String>,
    pub content: Option<String>,
}

#[post("/user")]
pub async fn create_user(state: Data<AppState>, body: Json<CreateUserBody>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(CreateUser {
            first_name: body.first_name.clone(),
            last_name: body.last_name.clone(),
            username: body.username.clone(),
            email: body.email.clone(),
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

#[get("/notes")]
pub async fn fetch_all_notes(state: Data<AppState>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(FetchAllNotes).await {
        Ok(Ok(notes)) => HttpResponse::Ok().json(notes),
        Ok(Err(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({ "message": "No notes found" }))
        }
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve notes" })),
    }
}

#[get("/user/{id}/notes")]
pub async fn fetch_user_notes(state: Data<AppState>, path: Path<i32>) -> impl Responder {
    let id: i32 = path.into_inner();

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(FetchUserNotes { user_id: id }).await {
        Ok(Ok(notes)) => HttpResponse::Ok().json(notes),
        Ok(Err(_)) => HttpResponse::NotFound()
            .json(serde_json::json!({ "message": format!("No notes for user {id}") })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve user notes" })),
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

    let created_on: DateTime<Utc> = Utc::now();

    match db
        .send(CreateNote {
            title: body.title.to_string(),
            content: body.content.to_string(),
            created_by: id,
            created_on,
        })
        .await
    {
        Ok(Ok(note)) => HttpResponse::Ok().json(note),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to create note" })),
    }
}

#[patch("/user/{id}/note/{note_id}")]
pub async fn update_user_note(
    state: Data<AppState>,
    path: Path<(i32, i32)>,
    body: Json<UpdateNoteBody>,
) -> impl Responder {
    let (user_id, note_id) = path.into_inner();

    let db: Addr<DbActor> = state.as_ref().db.clone();

    let existing_note = match db.send(FetchUserNotes { user_id }).await {
        Ok(Ok(notes)) => notes.into_iter().find(|note| note.id == note_id),
        _ => None,
    };

    if let Some(note) = existing_note {
        let updated_title: String = body.title.clone().unwrap_or(note.title);
        let updated_content: String = body.content.clone().unwrap_or(note.content);

        match db
            .send(UpdateNote {
                id: note_id,
                title: updated_title,
                content: updated_content,
                created_by: user_id,
            })
            .await
        {
            Ok(Ok(updated_note)) => HttpResponse::Ok().json(updated_note),
            Ok(Err(_)) => HttpResponse::NotFound()
                .json(serde_json::json!({ "message": format!("Note {note_id} not found") })),
            _ => HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to update note" })),
        }
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({ "message": format!("Note {note_id} not found") }))
    }
}

#[delete("/user/{id}/note/{note_id}")]
pub async fn delete_user_notes(state: Data<AppState>, path: Path<(i32, i32)>) -> impl Responder {
    let (_, note_id) = path.into_inner();

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(DeleteNote { note_id }).await {
        Ok(Ok(rows_affected)) if rows_affected > 0 => HttpResponse::Ok()
            .json(serde_json::json!({ "message": format!("Deleted note {note_id}") })),
        Ok(_) => HttpResponse::NotFound()
            .json(serde_json::json!({ "message": format!("Note {note_id} not found") })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to delete note" })),
    }
}
