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
use crate::{client::TelegramClient, env::LOGS_PATH, message::TelegramMessage, state::AppState};
use anyhow::{Context, Result};
use grammers_client::InputMessage;
use proc_macros::{check_in_group, check_senders};
use tokio::fs;

pub const PATTERN: &str = "/logs";

#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    {
        let metadata = fs::metadata(LOGS_PATH).await;
        if metadata.is_err()
            || (metadata.is_ok() && du::get_size(LOGS_PATH).context("failed to get dir size")? == 0)
        {
            let response = "Logs not found.";
            message.respond(response).await.context(response)?;

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

async fn send_log_zip(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    const ZIP_PATH: &str = "./logs.zip";

    zip_dir(LOGS_PATH, ZIP_PATH).await?;

    let file = fs::File::open(ZIP_PATH)
        .await
        .context("failed to open logs zip file")?;

    let size = file
        .metadata()
        .await
        .context("failed to get logs zip file metadata")?
        .len();

    message
        .respond(format!(
            "Sending logs sized {:.2}MB...\nThis may take a while.",
            size as f32 / 1024.0 / 1024.0
        ))
        .await?;

    let file = telegram_bot.upload_file(ZIP_PATH).await.context("logs")?;

    message.respond(InputMessage::default().file(file)).await?;

    std::fs::remove_file(ZIP_PATH).context("failed to remove file")?;

    Ok(())
}

async fn clear_logs(message: TelegramMessage) -> Result<()> {
    while let Some(entry) = fs::read_dir(LOGS_PATH)
        .await
        .context("failed to read logs dir")?
        .next_entry()
        .await
        .context("failed to read next entry in logs dir")?
    {
        let file_path = entry.path();
        fs::remove_file(&file_path)
            .await
            .context("failed to remove log file")
            .context(file_path.to_string_lossy().to_string())?;
    }

    let response = "Logs cleared.";
    message.respond(response).await.context(response)?;

    Ok(())
}
