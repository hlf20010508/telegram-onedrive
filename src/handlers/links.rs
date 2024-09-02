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

use super::utils::{
    cmd_parser, get_message_info, get_message_link, get_tg_file_size, preprocess_tg_file_name,
    upload_thumb,
};
use crate::env::BYPASS_PREFIX;
use crate::error::{Error, Result};
use crate::message::{ChatEntity, MessageInfo, TelegramMessage};
use crate::state::AppState;
use crate::tasker::CmdType;

pub const PATTERN: &str = "/links";

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let cmd = cmd_parser(message.text());

    if cmd.len() == 3 {
        let telegram_user = &state.telegram_user;
        let onedrive = &state.onedrive;
        let task_session = state.task_session.clone();

        let chat_user = telegram_user
            .get_chat(&ChatEntity::from(message.chat()))
            .await?;

        let link_head = &cmd[1];
        let link_num = cmd[2]
            .parse::<usize>()
            .map_err(|e| Error::new_parse_int(e, "failed to parse link number"))?;

        let MessageInfo {
            chat_entity,
            id: head_message_id,
        } = get_message_info(link_head)?;

        let chat_origin = telegram_user.get_chat(&chat_entity).await?;

        for offset in 0..link_num {
            let message_origin_id = head_message_id + offset as i32;

            let link = get_message_link(&chat_entity, message_origin_id);

            let Ok(message_origin) = telegram_user
                .get_message(&chat_origin, message_origin_id)
                .await
            else {
                message.reply(format!("message {} not found", link)).await?;

                continue;
            };

            let media = message_origin
                .media()
                .ok_or_else(|| Error::new("message does not contain any media"))?;

            let filename = preprocess_tg_file_name(&media);

            let total_length = get_tg_file_size(&media);

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
                        &chat_user,
                        InputMessage::text(response.as_str()).photo(uploaded),
                    )
                    .await
                    .context("linked message with thumb")
                    .details(response)?
                    .id(),
                None => telegram_user
                    .send_message(&chat_user, response.as_str())
                    .await
                    .context("linked message without thumn")
                    .details(response)?
                    .id(),
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
            let chat_user_hex = chat_user.pack().to_hex();
            let chat_origin_hex = message_origin.chat().pack().to_hex();

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
        }

        telegram_user
            .get_message(chat_user, message.id())
            .await?
            .delete()
            .await?;

        Ok(())
    } else {
        Err(Error::new("Unknown command for /links"))
    }
}
