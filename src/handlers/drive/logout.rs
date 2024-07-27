/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::client::OneDriveClient;
use crate::error::{Error, Result, ResultExt};
use crate::message::TelegramMessage;

pub async fn logout_current_drive(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
) -> Result<()> {
    let current_username = onedrive
        .get_current_username()
        .await?
        .ok_or_else(|| Error::new("no onedrive account is logged in"))?;

    onedrive.logout(None).await?;

    let response = {
        let mut response = format!(
            "OneDrive account {} logged out successfully.",
            current_username
        );

        if let Some(current_username) = onedrive.get_current_username().await? {
            response.push_str(&format!("\n\nCurrent account is {}", current_username));
        }

        response
    };

    message.respond(response.as_str()).await.details(response)?;

    Ok(())
}

pub async fn logout_drive(
    onedrive: &OneDriveClient,
    message: TelegramMessage,
    index: usize,
) -> Result<()> {
    let usernames = onedrive.get_usernames().await?;

    let selected_username = usernames
        .get(index)
        .ok_or_else(|| Error::new("account index out of range"))?;

    onedrive.logout(Some(selected_username.clone())).await?;

    let response = {
        let mut response = format!(
            "OneDrive account {} logged out successfully.",
            selected_username
        );

        if let Some(current_username) = onedrive.get_current_username().await? {
            response.push_str(&format!("\n\nCurrent account is {}", current_username));
        }

        response
    };

    message.respond(response.as_str()).await.details(response)?;

    Ok(())
}
