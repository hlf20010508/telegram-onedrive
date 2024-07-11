/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use onedrive_api::option::DriveItemPutOption;
use onedrive_api::{ConflictBehavior, ItemLocation, UploadSession, UploadSessionMeta};
use path_slash::PathBufExt;
use std::path::Path;

use super::OneDriveClient;
use crate::error::{Error, Result};

impl OneDriveClient {
    pub async fn multipart_upload_session_builder(
        &self,
        root_path: &str,
        filename: &str,
    ) -> Result<(UploadSession, UploadSessionMeta)> {
        let file_path_obj = Path::new(root_path).join(filename);
        let file_path = file_path_obj.to_slash_lossy();

        let item_location = ItemLocation::from_path(&file_path)
            .ok_or_else(|| Error::new("file path does not start with /"))?;

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
            .map_err(|e| Error::context(e, "failed to create upload session"))?;

        Ok(session)
    }
}