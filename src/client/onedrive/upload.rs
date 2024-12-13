/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::OneDriveClient;
use anyhow::{anyhow, Context, Result};
use onedrive_api::{
    option::DriveItemPutOption, ConflictBehavior, ItemLocation, UploadSession, UploadSessionMeta,
};
use path_slash::PathBufExt;
use std::path::Path;

impl OneDriveClient {
    pub async fn multipart_upload_session_builder(
        &self,
        root_path: &str,
        filename: &str,
    ) -> Result<(UploadSession, UploadSessionMeta)> {
        let file_path_obj = Path::new(root_path).join(filename);
        let file_path = file_path_obj.to_slash_lossy();

        let item_location = ItemLocation::from_path(&file_path)
            .ok_or_else(|| anyhow!("file path does not start with /"))?;

        self.refresh_access_token().await?;

        let session = self
            .client
            .read()
            .await
            .new_upload_session_with_option(
                item_location,
                DriveItemPutOption::new().conflict_behavior(ConflictBehavior::Rename),
            )
            .await
            .context("failed to create upload session")?;

        tracing::debug!("built upload session for {}", filename);

        Ok(session)
    }
}
