/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{utils::validate_root_path, OneDriveClient};
use anyhow::Result;

impl OneDriveClient {
    pub async fn get_root_path(&self, should_consume_temp: bool) -> Result<String> {
        let temp_root_path_read = self.temp_root_path.read().await;
        let temp_root_path_exists = self.does_temp_root_path_exist().await;

        let root_path = if should_consume_temp && temp_root_path_exists {
            tracing::debug!("get root path from temp and should be consumed");

            let temp_root_path = temp_root_path_read.clone();
            self.clear_temp_root_path().await?;

            temp_root_path
        } else if !should_consume_temp && temp_root_path_exists {
            tracing::debug!("get root path from temp and should not be consumed");

            temp_root_path_read.clone()
        } else {
            tracing::debug!("get root path");

            self.session.read().await.root_path.clone()
        };

        tracing::debug!("got root path: {}", root_path);

        validate_root_path(&root_path)?;

        Ok(root_path)
    }

    pub async fn does_temp_root_path_exist(&self) -> bool {
        let is_exist = !self.temp_root_path.read().await.is_empty();

        tracing::debug!("onedrive temp root path exists: {}", is_exist);

        is_exist
    }

    pub async fn set_root_path(&self, path: &str) -> Result<()> {
        validate_root_path(path)?;

        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path = path.to_string();
        session.save().await?;

        tracing::info!("set onedrive root path: {}", path);

        Ok(())
    }

    pub async fn reset_root_path(&self) -> Result<()> {
        tracing::info!("reset onedrive root path to default");
        tracing::debug!("default root path: {}", self.default_root_path);

        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path.clone_from(&self.default_root_path);
        session.save().await?;

        tracing::debug!(
            "reset onedrive root path to default: {}",
            self.default_root_path
        );

        Ok(())
    }

    pub async fn set_temp_root_path(&self, path: &str) -> Result<()> {
        validate_root_path(path)?;

        *self.temp_root_path.write().await = path.to_string();

        tracing::info!("onedrive temp root path: {}", path);

        Ok(())
    }

    pub async fn clear_temp_root_path(&self) -> Result<()> {
        tracing::info!("clear onedrive temp root path");

        self.set_temp_root_path("").await
    }
}
