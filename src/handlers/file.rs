/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::utils::upload::upload_thumb;
use crate::{
    env::BYPASS_PREFIX,
    error::{Error, Result},
    handlers::utils::{get_tg_file_size, preprocess_tg_file_name},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
    tasker::{CmdType, TaskAborter},
};
use grammers_client::{types::Media, InputMessage};
use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};
use std::sync::Arc;

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
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
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let mut message_id = message.id();
    let mut message_id_forward = None;

    let cmd_type = match media {
        Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => CmdType::File,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    // if message is forwarded, or is grouped in a album, send its file name and thumb if exists
    // so that information of uploading successful can be showed
    if message_user.forward_header().is_some() || message_user.raw.grouped_id().is_some() {
        message_id_forward = Some(message_id);

        let uploaded = match media {
            Media::Photo(file) => upload_thumb(telegram_user, file.thumbs()).await?,
            Media::Document(file) => upload_thumb(telegram_user, file.thumbs()).await?,
            Media::Sticker(file) => upload_thumb(telegram_user, file.document.thumbs()).await?,
            _ => Err(Error::new(
                "media type is not one of photo, document and sticker",
            ))?,
        };

        let response = format!("{}{}", BYPASS_PREFIX, filename);
        match uploaded {
            Some(uploaded) => {
                message_id = telegram_user
                    .send_message(
                        &chat_user,
                        InputMessage::text(response.as_str()).photo(uploaded),
                    )
                    .await
                    .context("forwarded message with thumb")
                    .details(response)?
                    .id();
            }
            None => {
                message_id = telegram_user
                    .send_message(&chat_user, response.as_str())
                    .await
                    .context("forwarded message without thumn")
                    .details(response)?
                    .id();
            }
        }
    }

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

    let mut aborters = state.task_session.aborters.write().await;

    let id = task_session
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
            None,
            message_id,
            message_id_forward,
            None,
        )
        .await?;

    let aborter = Arc::new(TaskAborter::new(id, &filename));
    let chat_id = chat_user.id();

    // insert both message_id and message_id_forward so that both of them can be used to abort the task
    aborters.insert((chat_id, message_id), (aborter.clone(), message_id_forward));

    if let Some(message_id_forward) = message_id_forward {
        aborters.insert((chat_id, message_id_forward), (aborter, Some(message_id)));
    }

    tracing::info!("inserted file task: {} size: {}", filename, total_length);

    Ok(())
}
