use super::messages::*;
use crate::utils::{
    db::{AppState, DbActor},
    jwt::Claims,
};
use actix::Addr;
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize)]
pub struct NoteQuery {
    search: Option<String>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}
#[utoipa::path(
    path = "/admin/api/notes",
    params(
        ("search" = String, Query, description = "Search terms to seach notes"),
        ("sort_field" = String, Query, description = "Provide title or content"),
        ("sort_order" = String, Query, description = "Provide asc or desc"),
        ("limit" = String, Query, description = "How much data to display"),
        ("offset" = String, Query, description = "Data to be skip"),
    ),
    responses(
        (status = 200, description = "Get all notes"),
        (status = 404, description = "No notes found"),
        (status = 500, description = "Unable to retrieve notes"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
#[get("/notes")]
pub async fn fetch_notes(state: Data<AppState>, query: Query<NoteQuery>) -> impl Responder {
    let db: Addr<DbActor> = state.as_ref().db.clone();

    match db
        .send(FetchNotes {
            search: query.search.clone(),
            sort_field: query.sort_field.clone(),
            sort_order: query.sort_order.clone(),
            limit: query.limit.clone(),
            offset: query.offset.clone(),
        })
        .await
    {
        Ok(Ok(notes)) => HttpResponse::Ok().json(notes),
        Ok(Err(_)) => {
            HttpResponse::NotFound().json(serde_json::json!({ "message": "No notes found" }))
        }
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Unable to retrieve notes" })),
    }
}

#[utoipa::path(
    path = "/secure/api/user/notes",
    responses(
        (status = 200, description = "Get my notes"),
        (status = 401, description = "Bearer auth required"),
        (status = 500, description = "Failed to create note"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

#[derive(Deserialize, ToSchema)]
pub struct CreateNoteBody {
    #[schema(example = "My Note", required = true)]
    pub title: String,
    #[schema(example = "This is my note", required = true)]
    pub content: String,
}

#[utoipa::path(
    path = "/secure/api/user/note",
    request_body = CreateNoteBody,
    responses(
        (status = 200, description = "Create a new user", body = CreateNote),
        (status = 401, description = "Bearer auth required"),
        (status = 500, description = "Failed to create note"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
    let updated_on: DateTime<Utc> = Utc::now();

    match db
        .send(CreateNote {
            title: body.title.to_string(),
            content: body.content.to_string(),
            created_by: claims.id,
            created_on,
            updated_on,
        })
        .await
    {
        Ok(Ok(note)) => HttpResponse::Ok().json(note),
        _ => HttpResponse::InternalServerError()
            .json(serde_json::json!({ "message": "Failed to create note" })),
    }
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateNoteBody {
    #[schema(example = "My Note")]
    pub title: Option<String>,
    #[schema(example = "My Note")]
    pub content: Option<String>,
}

#[utoipa::path(
    path = "/secure/api/user/note/update/{note_id}",
    request_body = UpdateNoteBody,
    responses(
        (status = 200, description = "Update successful"),
        (status = 401, description = "Bearer auth required"),
        (status = 500, description = "Failed to update note"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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

        let updated_on: DateTime<Utc> = Utc::now();

        match db
            .send(UpdateNote {
                id: note_id,
                title: updated_title,
                content: updated_content,
                created_by: claims.id,
                updated_on,
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

#[utoipa::path(
    path = "/secure/api/user/note/delete/{note_id}",
    responses(
        (status = 200, description = "Delete note successful"),
        (status = 401, description = "Bearer auth required"),
        (status = 500, description = "Failed to delete note"),
    ),
    security(
        ("bearer_auth" = [])
    )
)]
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
