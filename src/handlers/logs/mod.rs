/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod clear;
mod docs;
mod send;

use super::utils::cmd_parser;
use crate::{
    env::LOGS_PATH,
    error::{Error, Result},
    message::TelegramMessage,
    state::AppState,
};
use clear::clear_logs;
use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace, check_in_group, check_senders};
use send::send_log_zip;
use tokio::fs;

pub const PATTERN: &str = "/logs";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    {
        let metadata = fs::metadata(LOGS_PATH).await;
        if metadata.is_err()
            || (metadata.is_ok()
                && du::get_size(LOGS_PATH)
                    .map_err(|e| Error::new("failed to get dir size").raw(e))?
                    == 0)
        {
            let response = "Logs not found.";
            message.respond(response).await.details(response)?;

            return Ok(());
        }
    }

    let telegram_bot = &state.telegram_bot;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /logs
        send_log_zip(telegram_bot, message).await?;
    } else if cmd.len() == 2 && cmd[1] == "clear" {
        // /logs clear
        clear_logs(message).await?;
    } else {
        message
            .reply(InputMessage::html(format!(
                "Unknown command for /logs\n{}",
                docs::USAGE
            )))
            .await?;
    }

    Ok(())
}
