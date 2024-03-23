use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use eyre::WrapErr;
use tokio::fs::{self, File};

use crate::{error::DoorstepError, image::image_to_bitmap, state::AppState};

#[tracing::instrument(skip(app_state))]
pub async fn backgrounds_list(State(app_state): State<AppState>) -> impl IntoResponse {
    let backgrounds_dir = app_state.config.backgrounds_dir();
    let mut files = Vec::new();

    let mut readdir = fs::read_dir(backgrounds_dir)
        .await
        .wrap_err("Failed to read backgrounds directory")?;

    while let Some(entry) = readdir
        .next_entry()
        .await
        .wrap_err("Failed to read directory entry")?
    {
        let metadata = entry.metadata().await.wrap_err("Failed to read metadata")?;
        if metadata.is_file() {
            // add filename along with its modified time
            files.push((
                entry.file_name().to_string_lossy().to_string(),
                metadata
                    .modified()
                    .wrap_err("Failed to read modified time")?
                    .to_owned(),
            ));
        }
    }

    Ok::<_, DoorstepError>((StatusCode::OK, Json(files)))
}

/// Update the background image
#[tracing::instrument(skip(app_state, multipart))]
pub async fn update_background(
    State(app_state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut name = None;
    let mut bytes = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .wrap_err("Failed to read field")?
    {
        if field.name() != Some("background") {
            continue;
        }

        name = field.file_name().map(ToString::to_string);
        if name.is_none() {
            return Ok((StatusCode::BAD_REQUEST, "No file name provided"));
        }

        let mut stream = field
            .bytes()
            .await
            .wrap_err("Failed to read field bytes")?
            .to_vec();
        bytes.append(&mut stream);
    }

    let name = name.ok_or_else(|| "No file name provided")?;
    let file_path = app_state.config.backgrounds_dir().join(name);
    let mut file = File::create(file_path)
        .await
        .wrap_err("Failed to create file")?;

    tokio::io::copy(&mut &*bytes, &mut file)
        .await
        .wrap_err("Failed to write file")?;

    let target_width = app_state.config.background_width();
    let target_height = app_state.config.background_height();
    let bytes =
        tokio::task::spawn_blocking(move || image_to_bitmap(&bytes, target_width, target_height))
            .await
            .wrap_err("Failed to spawn blocking task")?
            .wrap_err("Failed to convert new background image")?;
    let mut background = app_state.background.lock().await;
    *background = Some(bytes);

    Ok::<_, DoorstepError>((StatusCode::OK, "Background updated successfully"))
}

/// Get the current background image
///
/// If no background image is set, this will return a 404
#[tracing::instrument(skip(app_state))]
pub async fn get_background(State(app_state): State<AppState>) -> Response {
    let background = app_state.background.lock().await;
    let background = match &*background {
        Some(bytes) => bytes.clone(),
        None => return (StatusCode::NOT_FOUND, "No background set").into_response(),
    };

    (StatusCode::OK, background).into_response()
}