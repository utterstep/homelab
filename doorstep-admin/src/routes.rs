use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Multipart, Query, State},
    http::StatusCode,
    response::{AppendHeaders, IntoResponse, Json},
};
use eyre::WrapErr;
use serde::Deserialize;
use tracing::{debug, info};

use crate::{controllers, error::DoorstepError, state::AppState};

#[tracing::instrument(skip(app_state))]
pub async fn backgrounds_list(State(app_state): State<AppState>) -> impl IntoResponse {
    let files = controllers::list_backgrounds(app_state)
        .await
        .wrap_err("Failed to list backgrounds")?
        .into_iter()
        .map(|background_file| {
            Ok((
                background_file.name().to_owned(),
                background_file
                    .metadata()
                    .modified()
                    .wrap_err("Failed to read modified time")?
                    .to_owned()
                    .duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .wrap_err("Failed to calculate duration since epoch")?,
            ))
        })
        .collect::<Result<Vec<_>, DoorstepError>>()
        .wrap_err("Failed to extract timestamps from files meta")?;

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

        name = Some(
            field
                .file_name()
                .map(ToString::to_string)
                .ok_or_else(|| DoorstepError::InvalidRequest("No filename provided".to_owned()))?,
        );

        let mut stream = field
            .bytes()
            .await
            .wrap_err("Failed to read field bytes")?
            .to_vec();
        bytes.append(&mut stream);
    }

    let name =
        name.ok_or_else(|| DoorstepError::InvalidRequest("No background provided".to_owned()))?;

    let hash = controllers::update_background(&name, bytes, app_state.clone())
        .await
        .wrap_err("Failed to update background")?;

    let headers = AppendHeaders([("X-Background-Hash", hash)]);

    Ok::<_, DoorstepError>((StatusCode::OK, headers, "Background updated successfully"))
}

#[derive(Debug, Deserialize)]
pub struct BackgroundRequestQuery {
    known_hash: Option<u32>,
}

/// Get the current background image
///
/// If no background image is set, this will return a 404
#[tracing::instrument(skip(app_state))]
pub async fn get_background(
    State(app_state): State<AppState>,
    Query(query): Query<BackgroundRequestQuery>,
) -> impl IntoResponse {
    let background = controllers::get_background(app_state)
        .await
        .wrap_err("Failed to get background")?;
    let current_hash = background.hash();

    let headers = AppendHeaders([("X-Background-Hash", current_hash)]);

    if let Some(known_hash) = query.known_hash {
        debug!(
            hash = current_hash,
            known_hash, "Checking if background is modified"
        );
        if current_hash == known_hash {
            info!(hash = current_hash, "Background not modified");
            return Ok(
                (StatusCode::NOT_MODIFIED, headers, "Background not modified").into_response(),
            );
        }
    }

    Ok::<_, DoorstepError>((StatusCode::OK, headers, background.bytes().to_vec()).into_response())
}
