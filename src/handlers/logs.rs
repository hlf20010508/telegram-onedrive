/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    docs::format_help,
    utils::{text::cmd_parser, zip::zip_dir},
};
use crate::{
    client::TelegramClient,
    env::LOGS_PATH,
    error::{Error, Result},
    message::TelegramMessage,
    state::AppState,
};
use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace, check_in_group, check_senders};
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
    } else if cmd.len() == 2 && cmd[1] == "help" {
        // /logs help
        message
            .respond(InputMessage::html(format_help(PATTERN)))
            .await
            .context("help")?;
    } else {
        message
            .reply(InputMessage::html(format_help(PATTERN)))
            .await?;
    }

    Ok(())
}

#[add_context]
#[add_trace]
async fn send_log_zip(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    const ZIP_PATH: &str = "./logs.zip";

    message.respond("Sending logs...").await?;

    zip_dir(LOGS_PATH, ZIP_PATH)?;

    let file = telegram_bot.upload_file(ZIP_PATH).await.context("logs")?;

    message.respond(InputMessage::default().file(file)).await?;

    std::fs::remove_file(ZIP_PATH).map_err(|e| Error::new("failed to remove file").raw(e))?;

    Ok(())
}

#[add_context]
#[add_trace]
async fn clear_logs(message: TelegramMessage) -> Result<()> {
    while let Some(entry) = fs::read_dir(LOGS_PATH)
        .await
        .map_err(|e| Error::new("failed to read logs dir").raw(e))?
        .next_entry()
        .await
        .map_err(|e| Error::new("failed to read next entry in logs dir").raw(e))?
    {
        fs::remove_file(entry.path()).await.map_err(|e| {
            Error::new("failed to remove log file")
                .raw(e)
                .details(entry.path().to_string_lossy())
        })?;
    }

    let response = "Logs cleared.";
    message.respond(response).await.details(response)?;

    Ok(())
}
