use std::{
    cmp::Reverse,
    fs::Metadata,
    path::{Path, PathBuf},
};

use eyre::WrapErr;
use tokio::fs::File;
use tracing::debug;
use xxhash_rust::xxh32::xxh32;

use crate::{
    config::HASH_SEED,
    error::DoorstepError,
    image::image_to_bitmap,
    state::{AppState, BackgroundState},
};

#[tracing::instrument(skip(bytes, app_state), fields(image_size = bytes.len()), err)]
pub(crate) async fn update_background(
    name: &str,
    bytes: Vec<u8>,
    app_state: &AppState,
) -> Result<u32, DoorstepError> {
    let file_path = app_state.config.backgrounds_dir().join(name);
    let mut file = File::create(&file_path)
        .await
        .wrap_err("Failed to create file")?;

    debug!(file_path = ?file_path, "Writing new background image");
    tokio::io::copy(&mut &*bytes, &mut file)
        .await
        .wrap_err("Failed to write file")?;

    debug!("Converting new background image to greyscale 4-bit bitmap");
    let target_width = app_state.config.background_width();
    let target_height = app_state.config.background_height();
    let bytes =
        tokio::task::spawn_blocking(move || image_to_bitmap(&bytes, target_width, target_height))
            .await
            .wrap_err("Failed to spawn blocking task")?
            .wrap_err("Failed to convert new background image")?;
    let hash = xxh32(&bytes, HASH_SEED);
    debug!(hash, "New background image written, saving to state");

    let mut background = app_state.background.write().await;
    *background = Some(BackgroundState::new(bytes, hash, name.to_owned()));

    Ok(hash)
}

#[derive(Debug)]
pub struct BackgroundFile {
    name: String,
    full_path: PathBuf,
    metadata: Metadata,
}

impl BackgroundFile {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn full_path(&self) -> &Path {
        &self.full_path
    }
}

#[tracing::instrument(skip(app_state), err)]
pub(crate) async fn list_backgrounds(
    app_state: &AppState,
) -> Result<Vec<BackgroundFile>, DoorstepError> {
    let backgrounds_dir = app_state.config.backgrounds_dir();
    let mut files = Vec::new();

    let mut readdir = tokio::fs::read_dir(backgrounds_dir)
        .await
        .wrap_err("Failed to read backgrounds directory")?;

    while let Some(entry) = readdir
        .next_entry()
        .await
        .wrap_err("Failed to read directory entry")?
    {
        let metadata = entry.metadata().await.wrap_err("Failed to read metadata")?;
        if metadata.is_file() {
            let full_path = entry
                .path()
                .canonicalize()
                .wrap_err("Failed to canonicalize path")?;

            // add filename along with its metadata
            files.push(BackgroundFile {
                name: entry.file_name().to_string_lossy().to_string(),
                full_path,
                metadata,
            });
        }
    }

    files.sort_unstable_by_key(|file| Reverse(file.metadata().modified().ok()));

    Ok(files)
}

#[derive(Debug)]
pub struct Background {
    bytes: Vec<u8>,
    hash: u32,
}

impl Background {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }
}

#[tracing::instrument(skip(app_state), err)]
pub(crate) async fn get_background(app_state: AppState) -> Result<Background, DoorstepError> {
    let background = app_state.background.read().await;

    match background.as_ref() {
        Some(background) => {
            let bytes = background.data().to_vec();
            let hash = background.hash();
            Ok(Background { bytes, hash })
        }
        None => Err(DoorstepError::NotFound("No background set".to_owned())),
    }
}
