/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod clear;
mod docs;
mod send;

use grammers_client::InputMessage;
use tokio::fs;

use clear::clear_logs;
use send::send_log_file;

use super::utils::cmd_parser;
use crate::message::TelegramMessage;
use crate::env::LOG_PATH;
use crate::error::{Result, ResultExt};
use crate::state::AppState;
use crate::{check_in_group, check_senders, check_tg_login};

pub const PATTERN: &str = "/logs";

pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_tg_login!(message, state);

    {
        let metadata = fs::metadata(LOG_PATH).await;
        if metadata.is_err() || (metadata.is_ok() && metadata.unwrap().len() == 0) {
            let response = "Logs not found.";
            message.respond(response).await.details(response)?;

            return Ok(());
        }
    }

    let telegram_bot = &state.telegram_bot;

    let cmd = cmd_parser(message.text());

    if cmd.len() == 1 {
        // /logs
        send_log_file(telegram_bot, message).await?;
    } else if cmd.len() == 2 && cmd[1] == "clear" {
        // /logs clear
        clear_logs(message).await?;
    } else {
        message
            .reply(InputMessage::html(format!(
                "Unknown command for /logs\n{}",
                docs::USAGE
            )))
            .await
            .context("command error for logs")?;
    }

    Ok(())
}
