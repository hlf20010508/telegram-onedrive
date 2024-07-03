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

pub async fn show_drive(onedrive: &OneDriveClient, message: Arc<Message>) -> Result<()> {
    let usernames = onedrive.get_usernames().await?;
    if let Some(current_username) = onedrive.get_current_username().await? {
        if usernames.len() > 0 {
            let response = {
                let mut response = format!("Current account is {}", current_username);

                if usernames.len() > 1 {
                    response.insert_str(0, "\n");
                    for i in (1..=usernames.len()).rev() {
                        response.insert_str(0, &format!("{}. {}\n", i, usernames[i - 1]));
                    }
                }

                response
            };
            message
                .respond(response.as_str())
                .await
                .map_err(|e| Error::respond_error(e, response))?;

            return Ok(());
        }
    }

    let response = "No account found.";
    message
        .respond(response)
        .await
        .map_err(|e| Error::respond_error(e, response))?;

    Ok(())
}
