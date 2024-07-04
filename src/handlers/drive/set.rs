/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;

use crate::client::OneDriveClient;
use crate::error::{Error, Result};

pub async fn set_drive(
    onedrive: &OneDriveClient,
    message: Arc<Message>,
    index: usize,
) -> Result<()> {
    let current_username = onedrive
        .get_current_username()
        .await?
        .ok_or_else(|| Error::new("no onedrive account is logged in"))?;

    let usernames = onedrive.get_usernames().await?;

    let selected_username = usernames
        .get(index)
        .ok_or_else(|| Error::new("account index out of range"))?;

    onedrive.change_account(selected_username).await?;

    if current_username != *selected_username {
        let response = format!(
            "Changed account from\n{}\nto\n{}",
            current_username, selected_username
        );
        message
            .respond(response.as_str())
            .await
            .map_err(|e| Error::respond_error(e, response))?;
    } else {
        let response = "Same account, nothing to change.";
        message
            .respond(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;
    }

    Ok(())
}
