/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use onedrive_api::{DriveLocation, OneDrive};
use proc_macros::add_trace;

use super::OneDriveClient;
use crate::error::Result;

impl OneDriveClient {
    #[add_trace(context)]
    pub async fn get_usernames(&self) -> Result<Vec<String>> {
        self.session.read().await.get_usernames().await
    }

    #[add_trace(context)]
    pub async fn get_current_username(&self) -> Result<Option<String>> {
        self.session.read().await.get_current_username().await
    }

    #[add_trace(context)]
    pub async fn change_account(&self, username: &str) -> Result<()> {
        let mut session = self.session.write().await;

        session.change_session(username).await?;

        *self.client.write().await =
            OneDrive::new(session.access_token.clone(), DriveLocation::me());

        Ok(())
    }
}
