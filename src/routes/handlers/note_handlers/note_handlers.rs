use super::messages::*;
use crate::utils::{
    db::{AppState, DbActor},
    jwt::Claims,
};
use actix::Addr;
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateNoteBody {
    pub title: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateNoteBody {
    pub title: Option<String>,
    pub content: Option<String>,
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

#[get("/user/notes")]
pub async fn fetch_user_notes(state: Data<AppState>, req: HttpRequest) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Unauthorized access" }));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db.send(FetchUserNotes { user_id: claims.id }).await {
        Ok(Ok(notes)) => HttpResponse::Ok().json(notes),
        Ok(Err(_)) => HttpResponse::NotFound()
            .json(serde_json::json!({ "message": format!("No notes for user {}", claims.id) })),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve user notes" })),
    }
}

#[post("/user/note")]
pub async fn create_user_notes(
    state: Data<AppState>,
    req: HttpRequest,
    body: Json<CreateNoteBody>,
) -> impl Responder {
    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({"message": "Unauthorized access"}));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    let created_on: DateTime<Utc> = Utc::now();

    match db
        .send(CreateNote {
            title: body.title.to_string(),
            content: body.content.to_string(),
            created_by: claims.id,
            created_on,
        })
        .await
    {
        Ok(Ok(note)) => HttpResponse::Ok().json(note),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to create note" })),
    }
}

#[patch("/user/note/update/{note_id}")]
pub async fn update_user_note(
    state: Data<AppState>,
    req: HttpRequest,
    path: Path<i32>,
    body: Json<UpdateNoteBody>,
) -> impl Responder {
    let note_id: i32 = path.into_inner();

    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Unauthorized access" }));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    let existing_note: Option<crate::models::Note> =
        match db.send(FetchUserNotes { user_id: claims.id }).await {
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
                created_by: claims.id,
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

#[delete("/user/note/delete/{note_id}")]
pub async fn delete_user_note(
    state: Data<AppState>,
    req: HttpRequest,
    path: Path<i32>,
) -> impl Responder {
    let note_id: i32 = path.into_inner();

    let claims: Claims = match req.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => {
            return HttpResponse::Unauthorized()
                .json(serde_json::json!({ "message": "Unauthorized access" }));
        }
    };

    let db: Addr<DbActor> = state.as_ref().db.clone();

    let existing_note: Option<crate::models::Note> =
        match db.send(FetchUserNotes { user_id: claims.id }).await {
            Ok(Ok(notes)) => notes.into_iter().find(|note| note.id == note_id),
            _ => None,
        };

    if let Some(_) = existing_note {
        match db.send(DeleteNote { note_id }).await {
            Ok(Ok(rows_affected)) if rows_affected > 0 => HttpResponse::Ok()
                .json(serde_json::json!({ "message": format!("Deleted note {}", note_id) })),
            Ok(_) => HttpResponse::NotFound()
                .json(serde_json::json!({ "message": format!("Note {} not found", note_id) })),
            _ => HttpResponse::InternalServerError()
                .json(serde_json::json!({ "message": "Failed to delete note" })),
        }
    } else {
        HttpResponse::NotFound()
            .json(serde_json::json!({ "message": format!("Note {} not found", note_id) }))
    }
}
