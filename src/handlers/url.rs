/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::{
        get_filename,
        text::{cmd_parser, TextExt},
    },
};
use crate::{
    error::{Error, Result},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
    tasker::CmdType,
    utils::get_http_client,
};
use grammers_client::InputMessage;
use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};
use reqwest::header;

pub const PATTERN: &str = "/url";

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let cmd = cmd_parser(message.text());

    if cmd.len() == 2 {
        if cmd[1] == "help" {
            // /url help
            message
                .respond(InputMessage::html(format_help(PATTERN)))
                .await
                .context("help")?;

            Ok(())
        } else {
            // /url $url
            let telegram_user = &state.telegram_user;
            let onedrive = &state.onedrive;
            let task_session = state.task_session.clone();

            let url = cmd[1].url_encode();

            if url.starts_with("http://") || url.starts_with("https://") {
                let http_client = get_http_client()?;

                let response = http_client
                    .head(&url)
                    .send()
                    .await
                    .map_err(|e| Error::new("failed to send head request for /url").raw(e))?;

                let filename = get_filename(&url, &response)?;

                let total_length = match response.headers().get(header::CONTENT_LENGTH) {
                Some(content_length) => content_length
                    .to_str()
                    .map_err(|e| Error::new( "header Content-Length has invisible ASCII chars").raw(e))?
                    .parse::<u64>()
                    .map_err(|e| Error::new( "failed to parse header Content-Length to u64").raw(e))?,
                None => return Err(Error::new(format!(
                    "Content-Length not found in response headers.\nStatus code:\n{}\nResponse headers:\n{:#?}",
                    response.status(),
                    response.headers()
                ))),
            };

                let root_path = onedrive.get_root_path(true).await?;

                let (upload_session, upload_session_meta) = onedrive
                    .multipart_upload_session_builder(&root_path, &filename)
                    .await?;

                let current_length = upload_session_meta
                    .next_expected_ranges
                    .first()
                    .map_or(0, |range| range.start);

                let chat_bot_hex = message.chat().pack().to_hex();
                let chat_user_hex = telegram_user
                    .get_chat(&ChatEntity::from(message.chat()))
                    .await?
                    .pack()
                    .to_hex();

                task_session
                    .insert_task(
                        CmdType::Url,
                        &filename,
                        &root_path,
                        Some(url),
                        upload_session.upload_url(),
                        current_length,
                        total_length,
                        &chat_bot_hex,
                        &chat_user_hex,
                        None,
                        message.id(),
                        None,
                        None,
                    )
                    .await?;

                tracing::info!("inserted url task: {} size: {}", filename, total_length);

                Ok(())
            } else {
                Err(Error::new("not an http url"))
            }
        }
    } else {
        message
            .reply(InputMessage::html(format_unknown_command_help(PATTERN)))
            .await?;

        Ok(())
    }
}
