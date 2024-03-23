use std::{fs, path::Path};

use eyre::{Result, WrapErr};
use tracing::{debug, info};

use crate::image::image_to_bitmap;

use super::AppState;

impl AppState {
    /// Try to load the latest background image from the backgrounds directory
    #[tracing::instrument(skip(self), err)]
    pub async fn try_init_background(&mut self) -> Result<()> {
        let backgrounds_dir = self.config.backgrounds_dir();
        let mut latest_file = None;

        // (1) Create the backgrounds directory if it doesn't exist
        fs::create_dir_all(backgrounds_dir).wrap_err("Failed to create backgrounds directory")?;

        debug!(
            ?backgrounds_dir,
            "Checking backgrounds directory for images"
        );

        // (2) Read the directory and find the latest file
        for entry in
            fs::read_dir(backgrounds_dir).wrap_err("Failed to read backgrounds directory")?
        {
            let entry = entry.wrap_err("Failed to read directory entry")?;
            let metadata = entry.metadata().wrap_err("Failed to read metadata")?;
            if metadata.is_file() {
                let modified = metadata
                    .modified()
                    .wrap_err("Failed to read modified time")?;
                if let Some((latest, _)) = latest_file {
                    if modified > latest {
                        latest_file = Some((modified, entry.path()));
                    }
                } else {
                    latest_file = Some((modified, entry.path()));
                }
            }
        }

        // (3) Read the latest file and convert it to a bitmap
        if let Some((_, path)) = latest_file {
            self.try_load_background(&path).await?;
        } else {
            info!("No background image found");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn try_load_background(&mut self, path: &Path) -> Result<()> {
        debug!(?path, "Loading existing background image");

        let width = self.config.background_width();
        let height = self.config.background_height();

        let bytes = tokio::fs::read(path)
            .await
            .wrap_err_with(|| format!("Failed to read specified background file: {path:?}"))?;
        let bytes = image_to_bitmap(&bytes, width, height)
            .wrap_err("Failed to convert stored background image")?;
        let mut background = self.background.lock().await;
        *background = Some(bytes);

        info!("Background image loaded successfully");

        Ok(())
    }
}
