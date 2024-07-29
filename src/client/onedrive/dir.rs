/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace};

use super::OneDriveClient;
use crate::error::Result;

impl OneDriveClient {
    #[add_context]
    #[add_trace]
    pub async fn get_root_path(&self, should_consume_temp: bool) -> Result<String> {
        let temp_root_path_read = self.temp_root_path.read().await;
        let temp_root_path_exists = self.does_temp_root_path_exist().await;

        let root_path = if should_consume_temp && temp_root_path_exists {
            let temp_root_path = temp_root_path_read.clone();
            self.clear_temp_root_path().await?;

            temp_root_path
        } else if !should_consume_temp && temp_root_path_exists {
            temp_root_path_read.clone()
        } else {
            self.session.read().await.root_path.clone()
        };

        Ok(root_path)
    }

    #[add_trace]
    pub async fn does_temp_root_path_exist(&self) -> bool {
        !self.temp_root_path.read().await.is_empty()
    }

    #[add_context]
    #[add_trace]
    pub async fn set_root_path(&self, path: &str) -> Result<()> {
        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path = path.to_string();
        session.save().await?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn reset_root_path(&self) -> Result<()> {
        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path.clone_from(&self.default_root_path);
        session.save().await?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn set_temp_root_path(&self, path: &str) -> Result<()> {
        *self.temp_root_path.write().await = path.to_string();

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn clear_temp_root_path(&self) -> Result<()> {
        self.set_temp_root_path("").await
    }
}
