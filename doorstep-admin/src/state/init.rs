use std::path::Path;

use eyre::{OptionExt, Result, WrapErr};
use tracing::{debug, info};

use crate::controllers;

use super::AppState;

impl AppState {
    /// Try to load the latest background image from the backgrounds directory
    #[tracing::instrument(skip(self), err)]
    pub async fn try_init_background(&mut self) -> Result<()> {
        let backgrounds = controllers::list_backgrounds(&*self).await?;
        let latest_file = backgrounds
            .into_iter()
            .max_by_key(|bg| bg.metadata().modified().ok());

        if let Some(background) = latest_file {
            self.try_load_background(background.full_path()).await?;
        } else {
            info!("No background image found");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), err)]
    pub async fn try_load_background(&mut self, path: &Path) -> Result<()> {
        debug!(?path, "Loading existing background image");

        let bytes = tokio::fs::read(path)
            .await
            .wrap_err_with(|| format!("Failed to read specified background file: {path:?}"))?;

        Ok(controllers::update_background(
            path.file_name()
                .ok_or_eyre("Trying to load background without filename")?
                .to_string_lossy()
                .as_ref(),
            bytes,
            &*self,
        )
        .await
        .map(|_| ())?)
    }
}
