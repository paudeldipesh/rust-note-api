use actix_web::HttpResponse;
use reqwest::{
    multipart::{Form, Part},
    Client,
};
use serde::Deserialize;
pub fn upload_image_validation(
    file_name: Option<String>,
    file_size: usize,
    max_file_size: u64,
) -> Result<(), HttpResponse> {
    match file_name {
        Some(name) => {
            if !name.ends_with(".png") && !name.ends_with(".jpg") {
                return Err(HttpResponse::BadRequest()
                    .json(serde_json::json!({ "message": "Invalid file type"})));
            }
        }
        None => {
            return Err(HttpResponse::BadRequest()
                .json(serde_json::json!({ "message": "File name is missing"})));
        }
    }

    match file_size {
        0 => {
            return Err(HttpResponse::BadRequest()
                .json(serde_json::json!({ "message": "Invalid file size"})));
        }
        length if length > max_file_size as usize => {
            return Err(HttpResponse::BadRequest()
                .json(serde_json::json!({ "message": "File size too long"})));
        }
        _ => {}
    }

    Ok(())
}

#[derive(Deserialize)]
struct CloudinaryResponse {
    secure_url: String,
}

pub async fn upload_image_to_cloudinary(
    temp_file_path: &std::path::Path,
    cloud_name: String,
    upload_preset: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client: Client = Client::new();

    let file: Vec<u8> = std::fs::read(temp_file_path)?;

    let part: Part = Part::bytes(file).file_name(
        temp_file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
    );

    let form: Form = Form::new()
        .part("file", part)
        .text("upload_preset", upload_preset);

    let url: String = format!(
        "https://api.cloudinary.com/v1_1/{}/image/upload",
        cloud_name
    );

    let response = client
        .post(url)
        .multipart(form)
        .send()
        .await?
        .json::<CloudinaryResponse>()
        .await?;

    Ok(response.secure_url)
}
