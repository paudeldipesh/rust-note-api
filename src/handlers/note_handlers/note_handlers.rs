use super::messages::*;
use crate::{
    models::Note,
    utils::{
        db::{AppState, DbActor},
        jwt::Claims,
    },
};
use actix::Addr;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path, Query},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use utoipa::ToSchema;

#[derive(Deserialize)]
pub struct NoteQuery {
    search: Option<String>,
    sort_field: Option<String>,
    sort_order: Option<String>,
    limit: Option<i64>,
    page: Option<i64>,
    active_status: Option<String>,
}

#[derive(Serialize)]
struct NotesResponse {
    total_notes: i64,
    number_of_page: i64,
    page: i64,
    notes: Vec<Note>,
}

#[utoipa::path(
    path = "/admin/api/notes",
    params(
        ("search" = Option<String>, Query, description = "Search term for filtering notes"),
        ("sort_field" = Option<String>, Query, description = "Field to sort by (e.g., title, content)"),
        ("sort_order" = Option<String>, Query, description = "Order to sort (asc or desc)"),
        ("page" = Option<i64>, Query, description = "Page number for pagination"),
        ("limit" = Option<i64>, Query, description = "Limit of notes per page"),
        ("active_status" = Option<String>, Query, description = "Filter by active status (active/inactive)")
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

    let active_status: Option<ActiveStatus> = match query.active_status.as_deref() {
        Some("active") => Some(ActiveStatus::Active),
        Some("inactive") => Some(ActiveStatus::Inactive),
        _ => None,
    };

    match db
        .send(FetchNotes {
            search: query.search.clone(),
            sort_field: query.sort_field.clone(),
            sort_order: query.sort_order.clone(),
            limit: query.limit.clone(),
            page: query.page.clone(),
            active_status,
        })
        .await
    {
        Ok(Ok((total_notes, notes, number_of_page, page))) => {
            HttpResponse::Ok().json(NotesResponse {
                total_notes,
                number_of_page,
                page,
                notes,
            })
        }
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

#[derive(Debug, MultipartForm)]
pub struct CreateNoteBody {
    title: Text<String>,
    content: Text<String>,
    image: Option<TempFile>,
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
    body: MultipartForm<CreateNoteBody>,
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

    if let Some(image) = &body.0.image.as_ref() {
        let file_name: Option<String> = image.file_name.clone();
        let file_size: usize = image.size;
        let max_file_size: u64 = 10485760;
        let temp_file_path: &std::path::Path = image.file.path();

        match &file_name {
            Some(name) => {
                if !name.ends_with(".png") && !name.ends_with(".jpg") {
                    return HttpResponse::BadRequest()
                        .json(serde_json::json!({ "message": "Invalid file type"}));
                }
            }
            None => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "message": "File name is missing"}));
            }
        }

        match file_size {
            0 => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "message": "Invalid file size"}));
            }
            length if length > max_file_size as usize => {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({ "message": "File size too long"}));
            }
            _ => {}
        }

        let time_stamp: i64 = Utc::now().timestamp();
        let mut file_path: PathBuf = PathBuf::from("./public");
        let new_file_name: String = format!("{}-{}", time_stamp, file_name.unwrap());
        file_path.push(&new_file_name);

        match std::fs::copy(temp_file_path, file_path) {
            Ok(_) => {
                println!("Image uploaded on the server");
                std::fs::remove_file(temp_file_path).unwrap_or_default();
            }
            Err(_) => println!("Unable to upload"),
        }

        match db
            .send(CreateNote {
                title: body.title.clone(),
                content: body.content.clone(),
                created_by: claims.id,
                created_on,
                updated_on,
            })
            .await
        {
            Ok(Ok(note)) => return HttpResponse::Ok().json(note),
            _ => {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "message": "Failed to create note" }))
            }
        }
    } else {
        match db
            .send(CreateNote {
                title: body.title.clone(),
                content: body.content.clone(),
                created_by: claims.id,
                created_on,
                updated_on,
            })
            .await
        {
            Ok(Ok(note)) => return HttpResponse::Ok().json(note),
            _ => {
                return HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "message": "Failed to create note" }))
            }
        }
    }
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateNoteBody {
    #[schema(example = "My Note")]
    pub title: Option<String>,
    #[schema(example = "My Note")]
    pub content: Option<String>,
    #[schema(example = true)]
    pub active: Option<bool>,
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
        let active_status: bool = body.active.clone().unwrap_or(true);

        let updated_on: DateTime<Utc> = Utc::now();

        match db
            .send(UpdateNote {
                id: note_id,
                title: updated_title,
                content: updated_content,
                active: active_status,
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
