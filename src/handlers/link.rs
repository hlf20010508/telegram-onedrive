/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Media;
use grammers_client::InputMessage;
use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};

use super::utils::upload_thumb;
use crate::client::TelegramClient;
use crate::env::BYPASS_PREFIX;
use crate::error::{Error, Result};
use crate::handlers::utils::{get_tg_file_size, preprocess_tg_file_name};
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::tasker::CmdType;

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let onedrive = &state.onedrive;
    let task_session = state.task_session.clone();

    let link = message.text();

    let message_origin = get_message_from_link(telegram_user, link).await?;

    let chat_user = telegram_user.get_chat(message.clone()).await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_origin
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let root_path = onedrive.get_root_path(true).await?;

    let (upload_session, upload_session_meta) = onedrive
        .multipart_upload_session_builder(&root_path, &filename)
        .await?;

    let current_length = upload_session_meta
        .next_expected_ranges
        .first()
        .map_or(0, |range| range.start);

    let chat_bot_hex = message.chat().pack().to_hex();
    let chat_user_hex = chat_user.pack().to_hex();
    let chat_origin_hex = message_origin.chat().pack().to_hex();

    let cmd_type = match media {
        Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => CmdType::Link,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    let uploaded = match media {
        Media::Photo(file) => upload_thumb(telegram_user, file.thumbs()).await?,
        Media::Document(file) => upload_thumb(telegram_user, file.thumbs()).await?,
        Media::Sticker(file) => upload_thumb(telegram_user, file.document.thumbs()).await?,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    let response = format!("{}{}\n\n{}", BYPASS_PREFIX, link, filename);
    let message_id = match uploaded {
        Some(uploaded) => telegram_user
            .send_message(
                chat_user,
                InputMessage::text(response.as_str()).photo(uploaded),
            )
            .await
            .context("linked message with thumb")
            .details(response)?
            .id(),
        None => telegram_user
            .send_message(chat_user, response.as_str())
            .await
            .context("linked message without thumn")
            .details(response)?
            .id(),
    };

    message_user.delete().await?;

    task_session
        .insert_task(
            cmd_type,
            &filename,
            &root_path,
            None,
            upload_session.upload_url(),
            current_length,
            total_length,
            &chat_bot_hex,
            &chat_user_hex,
            Some(chat_origin_hex),
            message_id,
            None,
            Some(message_origin.id()),
        )
        .await?;

    Ok(())
}

#[add_context]
#[add_trace]
async fn get_message_from_link(
    telegram_user: &TelegramClient,
    link: &str,
) -> Result<TelegramMessage> {
    let message = if let Some(message_info) = link.to_string().strip_prefix("https://t.me/c/") {
        // link from private group
        let message_info_vec = message_info
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if message_info_vec.len() == 2 {
            let chat_id = message_info_vec[0]
                .parse()
                .map_err(|e| Error::new_parse_int(e, "failed to parse chat id"))?;
            let message_id = message_info_vec[1]
                .parse()
                .map_err(|e| Error::new_parse_int(e, "failed to parse message id"))?;

            let chat = telegram_user.get_chat_from_id(chat_id).await?;

            telegram_user.get_message(chat, message_id).await?
        } else {
            return Err(Error::new("message info doesn't contain 2 elements"))?;
        }
    } else if let Some(message_info) = link.to_string().strip_prefix("https://t.me/") {
        // link from public group
        let message_info_vec = message_info
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        if message_info_vec.len() == 2 {
            let chat_name = &message_info_vec[0];
            let message_id = message_info_vec[1]
                .parse()
                .map_err(|e| Error::new_parse_int(e, "failed to parse message id"))?;

            let chat = telegram_user.get_chat_from_name(chat_name).await?;

            telegram_user.get_message(chat, message_id).await?
        } else {
            return Err(Error::new("message info doesn't contain 2 elements"))?;
        }
    } else {
        return Err(Error::new("not a message link"));
    };

    Ok(message)
}
