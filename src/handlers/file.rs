/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::Ordering;

use super::utils::upload::upload_thumb;
use crate::{
    handlers::utils::{get_tg_file_size, message::format_message_link, preprocess_tg_file_name},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
    tasker::{CmdType, InsertTask},
};
use anyhow::{anyhow, Context, Result};
use grammers_client::{types::Media, InputMessage};
use proc_macros::{check_in_group, check_od_login, check_senders, check_tg_login};

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let onedrive = &state.onedrive;
    let task_session = &state.task_session;

    let chat_user = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_user
        .media()
        .ok_or_else(|| anyhow!("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let message_id = message.id();

    let cmd_type = match media {
        Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => CmdType::File,
        _ => Err(anyhow!(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    let uploaded = match media {
        Media::Photo(file) => upload_thumb(state.clone(), file.thumbs()).await?,
        Media::Document(file) => upload_thumb(state.clone(), file.thumbs()).await?,
        Media::Sticker(file) => upload_thumb(state.clone(), file.document.thumbs()).await?,
        _ => Err(anyhow!(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    // in case if cancellation happens before inserting the task
    let _aborters = state.task_session.task_aborters.lock().await;

    let response = format_message_link(chat_user.id(), message_id, &filename);
    let message_indicator_id = match uploaded {
        Some(uploaded) => message
            .respond(InputMessage::html(&response).photo(uploaded))
            .await
            .context("message with thumb")
            .context(response)?
            .id(),
        None => message
            .respond(InputMessage::html(&response))
            .await
            .context("message without thumn")
            .context(response)?
            .id(),
    };

    let root_path = onedrive.get_root_path(true).await?;

    let (upload_session, upload_session_meta) = onedrive
        .multipart_upload_session_builder(&root_path, &filename)
        .await?;

    // all task should be new, so this should always be 0
    let current_length = upload_session_meta
        .next_expected_ranges
        .first()
        .map_or(0, |range| range.start);

    let chat_bot_hex = message.chat().pack().to_hex();
    let chat_user_hex = chat_user.pack().to_hex();

    let auto_delete = state.should_auto_delete.load(Ordering::Acquire);

    task_session
        .insert_task(InsertTask {
            cmd_type,
            filename: filename.clone(),
            root_path,
            url: None,
            upload_url: upload_session.upload_url().to_string(),
            current_length,
            total_length,
            chat_id: chat_user.id(),
            chat_bot_hex,
            chat_user_hex,
            chat_origin_hex: None,
            message_id,
            message_indicator_id,
            message_origin_id: None,
            auto_delete,
        })
        .await?;

    tracing::info!("inserted file task: {} size: {}", filename, total_length);

    Ok(())
}
