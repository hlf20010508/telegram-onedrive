/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::OneDriveClient;
use crate::error::Result;

impl OneDriveClient {
    pub async fn get_usernames(&self) -> Result<Vec<String>> {
        let session = self.session.read().await;

        session.get_usernames().await
    }

    pub async fn get_current_username(&self) -> Result<String> {
        let session = self.session.read().await;

        session.get_current_username().await
    }
}
