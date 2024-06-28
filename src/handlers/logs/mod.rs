/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use grammers_client::types::Message;
use grammers_client::InputMessage;
use std::sync::Arc;
use tokio::fs;

use super::utils::cmd_parser;
use crate::env::LOG_PATH;
use crate::error::{Error, Result};
use crate::state::AppState;
use crate::{check_in_group, check_senders, check_tg_login};

pub const PATTERN: &str = "/logs";

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_tg_login!(message, state);

    {
        let metadata = fs::metadata(LOG_PATH).await;
        if metadata.is_err() || (metadata.is_ok() && metadata.unwrap().len() == 0) {
            message.respond(docs::LOGS_NOT_FOUND).await.map_err(|e| {
                Error::context(e, "failed to respond message in logs for logs not found")
            })?;

            return Ok(());
        }
    }

    let telegram_bot = &state.telegram_bot;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /logs
        let file = telegram_bot
            .client
            .upload_file(LOG_PATH)
            .await
            .map_err(|e| Error::context(e, "failed to upload log file"))?;

        message
            .respond(InputMessage::default().file(file))
            .await
            .map_err(|e| Error::context(e, "failed to respond log file"))?;
    } else if cmd.len() == 2 && cmd[1] == "clear" {
        // /logs clear
        fs::remove_file(LOG_PATH)
            .await
            .map_err(|e| Error::context(e, "failed to remove log file"))?;

        message
            .respond(docs::LOGS_CLEARED)
            .await
            .map_err(|e| Error::context(e, "failed to respond message in logs for logs cleared"))?;
    } else {
        message
            .respond(InputMessage::html(docs::LOGS_FORMAT_WRONG))
            .await
            .map_err(|e| {
                Error::context(e, "failed to respond message in logs for unknown command")
            })?;
    }

    Ok(())
}
