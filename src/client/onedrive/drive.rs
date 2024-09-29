/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::OneDriveClient;
use crate::error::Result;
use onedrive_api::{DriveLocation, OneDrive};
use proc_macros::{add_context, add_trace};

impl OneDriveClient {
    #[add_context]
    #[add_trace]
    pub async fn get_usernames(&self) -> Result<Vec<String>> {
        self.session.read().await.get_usernames().await
    }

    #[add_context]
    #[add_trace]
    pub async fn get_current_username(&self) -> Result<Option<String>> {
        self.session.read().await.get_current_username().await
    }

    #[add_context]
    #[add_trace]
    pub async fn change_account(&self, username: &str) -> Result<()> {
        let mut session = self.session.write().await;

        session.change_session(username).await?;

        *self.client.write().await =
            OneDrive::new(session.access_token.clone(), DriveLocation::me());

        tracing::debug!("change account to {}", username);

        Ok(())
    }
}
