use super::messages::*;
use crate::handlers::note_handlers::utils::*;
use crate::{
    models::Note,
    utils::{
        self,
        db::{AppState, DbActor},
        jwt::Claims,
    },
};
use actix::Addr;
use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use actix_web::{
    delete, get, patch, post,
    web::{Data, Path, Query},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

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
    let created_on: NaiveDateTime = Utc::now().naive_local();
    let updated_on: NaiveDateTime = Utc::now().naive_local();

    if let Some(image) = &body.0.image.as_ref() {
        let file_name: Option<String> = image.file_name.clone();
        let file_size: usize = image.size;
        let max_file_size: u64 = 10485760;
        let temp_file_path: &std::path::Path = image.file.path();

        if let Err(err) = upload_image_validation(file_name, file_size, max_file_size) {
            return err;
        }

        let cloud_name: String = (*utils::constants::CLOUDINARY_CLOUD_NAME).clone();
        let upload_preset: String = (*utils::constants::CLOUDINARY_UPLOAD_PRESET).clone();

        let image_url =
            match upload_image_to_cloudinary(temp_file_path, cloud_name, upload_preset).await {
                Ok(url) => {
                    std::fs::remove_file(temp_file_path).unwrap_or_default();
                    Some(url)
                }
                Err(e) => {
                    println!("Error uploading image: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"message": "Failed to upload image"}));
                }
            };

        match db
            .send(CreateNote {
                title: body.title.clone(),
                content: body.content.clone(),
                created_by: claims.id,
                image_url,
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
                image_url: None,
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

#[derive(Debug, MultipartForm)]
pub struct UpdateNoteBody {
    pub title: Option<Text<String>>,
    pub content: Option<Text<String>>,
    pub active: Option<Text<bool>>,
    pub image: Option<TempFile>,
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
    body: MultipartForm<UpdateNoteBody>,
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
        let updated_title: String = body
            .0
            .title
            .as_ref()
            .map(|text| text.to_string())
            .unwrap_or(note.title);
        let updated_content: String = body
            .0
            .content
            .as_ref()
            .map(|text| text.to_string())
            .unwrap_or(note.content);
        let active_status: bool = body.0.active.as_ref().map(|text| text.0).unwrap_or(true);

        let mut updated_image_url: Option<String> = note.image_url.clone();

        if let Some(image) = &body.0.image.as_ref() {
            let file_name: Option<String> = image.file_name.clone();
            let file_size: usize = image.size;
            let max_file_size: u64 = 10485760;
            let temp_file_path: &std::path::Path = image.file.path();

            if let Err(err) = upload_image_validation(file_name, file_size, max_file_size) {
                return err;
            }

            let cloud_name: String = (*utils::constants::CLOUDINARY_CLOUD_NAME).clone();
            let upload_preset: String = (*utils::constants::CLOUDINARY_UPLOAD_PRESET).clone();

            match upload_image_to_cloudinary(temp_file_path, cloud_name, upload_preset).await {
                Ok(url) => {
                    std::fs::remove_file(temp_file_path).unwrap_or_default();
                    updated_image_url = Some(url);
                }
                Err(e) => {
                    println!("Error uploading image: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({ "message": "Failed to upload image" }));
                }
            };
        }

        let updated_on: NaiveDateTime = Utc::now().naive_local();

        match db
            .send(UpdateNote {
                id: note_id,
                title: updated_title,
                content: updated_content,
                image_url: updated_image_url,
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
