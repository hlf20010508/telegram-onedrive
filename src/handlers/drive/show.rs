/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::add_trace;

use crate::client::OneDriveClient;
use crate::error::Result;
use crate::message::TelegramMessage;

#[add_trace(context)]
pub async fn show_drive(onedrive: &OneDriveClient, message: TelegramMessage) -> Result<()> {
    let usernames = onedrive.get_usernames().await?;
    if let Some(current_username) = onedrive.get_current_username().await? {
        if !usernames.is_empty() {
            let response = {
                let mut response = format!("Current account is {}", current_username);

                if usernames.len() > 1 {
                    response.insert(0, '\n');
                    for i in (1..=usernames.len()).rev() {
                        response.insert_str(0, &format!("{}. {}\n", i, usernames[i - 1]));
                    }
                }

                response
            };
            message.respond(response.as_str()).await.details(response)?;

            return Ok(());
        }
    }

    let response = "No account found.";
    message.respond(response).await.details(response)?;

    Ok(())
}
