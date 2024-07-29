/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::media::Uploaded;
use grammers_client::types::photo_sizes::{PhotoSize, VecExt};
use grammers_client::types::{Downloadable, Media};
use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace};
use std::io::Cursor;

use crate::client::TelegramClient;
use crate::env::BYPASS_PREFIX;
use crate::error::{Error, Result};
use crate::handlers::utils::{get_tg_file_size, preprocess_tg_file_name};
use crate::message::TelegramMessage;
use crate::state::AppState;
use crate::tasker::CmdType;
use crate::{check_in_group, check_od_login, check_senders, check_tg_login};

#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_tg_login!(message, state);
    check_od_login!(message, state);

    let telegram_user = &state.telegram_user;
    let onedrive = &state.onedrive;
    let task_session = state.task_session.clone();

    let chat_user = telegram_user.get_chat(message.clone()).await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_user
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let filename = preprocess_tg_file_name(&media);

    let total_length = get_tg_file_size(&media);

    let root_path = onedrive.get_root_path(true).await?;

    let (upload_session, upload_session_meta) = onedrive
        .multipart_upload_session_builder(&root_path, &filename)
        .await?;

    let current_length = {
        match upload_session_meta.next_expected_ranges.first() {
            Some(range) => range.start,
            None => 0,
        }
    };

    let mut message_id = message.id();
    let chat_bot_hex = message.chat().pack().to_hex();
    let chat_user_hex = chat_user.pack().to_hex();

    let cmd_type = match media {
        Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => CmdType::File,
        _ => Err(Error::new(
            "media type is not one of photo, document and sticker",
        ))?,
    };

    let mut message_id_forward = None;

    if message_user.forward_header().is_some() {
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
                        chat_user,
                        InputMessage::text(response.as_str()).photo(uploaded),
                    )
                    .await
                    .context("forwarded message with thumb")
                    .details(response)?
                    .id();
            }
            None => {
                message_id = telegram_user
                    .send_message(chat_user, response.as_str())
                    .await
                    .context("forwarded message without thumn")
                    .details(response)?
                    .id()
            }
        }
    }

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
            message_id,
            message_id_forward,
        )
        .await?;

    Ok(())
}

async fn upload_thumb(client: &TelegramClient, thumbs: Vec<PhotoSize>) -> Result<Option<Uploaded>> {
    let uploaded = match thumbs.largest() {
        Some(thumb) => {
            let downloadable = Downloadable::PhotoSize(thumb.clone());
            let mut download = client.iter_download(&downloadable);

            let mut buffer = Vec::new();
            while let Some(chunk) = download.next().await.map_err(|e| {
                Error::new_telegram_invocation(e, "failed to download chunk for thumb")
            })? {
                buffer.extend(chunk);
            }

            let size = buffer.len();
            let mut stream = Cursor::new(buffer);
            let uploaded = client
                .upload_stream(&mut stream, size, "thumb.jpg".to_string())
                .await?;

            Some(uploaded)
        }
        None => None,
    };

    Ok(uploaded)
}
