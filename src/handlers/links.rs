/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::sync::atomic::Ordering;

use super::{
    docs::{format_help, format_unknown_command_help},
    utils::{
        get_tg_file_size,
        message::{get_message_info, get_message_link},
        preprocess_tg_file_name,
        text::cmd_parser,
        upload::upload_thumb,
    },
};
use crate::{
    env::BYPASS_PREFIX,
    error::{Error, ParserType, Result},
    message::{ChatEntity, MessageInfo, TelegramMessage},
    state::AppState,
    tasker::{CmdType, InsertTask},
};
use grammers_client::{types::Media, InputMessage};
use proc_macros::{
    add_context, add_trace, check_in_group, check_od_login, check_senders, check_tg_login,
};

pub const PATTERN: &str = "/links";

#[check_od_login]
#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    handle_links(message.clone(), message.text(), state, true).await
}

#[add_context]
#[add_trace]
pub async fn handle_links(
    message: TelegramMessage,
    text: &str,
    state: AppState,
    should_delete: bool,
) -> Result<()> {
    let cmd = cmd_parser(text);

    if cmd.len() == 2 && cmd[1] == "help" {
        // /links help
        message
            .respond(InputMessage::html(format_help(PATTERN)))
            .await
            .context("help")?;
    } else if cmd.len() == 3 {
        // /links $message_link $num
        let telegram_user = &state.telegram_user;
        let onedrive = &state.onedrive;
        let task_session = &state.task_session;

        let chat_user = telegram_user
            .get_chat(&ChatEntity::from(message.chat()))
            .await?;

        let link_head = &cmd[1];
        let link_num = cmd[2]
            .parse::<usize>()
            .map_err(|e| Error::new("failed to parse link number").raw(e))?;

        let MessageInfo {
            chat_entity,
            id: head_message_id,
        } = get_message_info(link_head)?;

        let chat_origin = telegram_user.get_chat(&chat_entity).await?;

        let auto_delete = state.should_auto_delete.load(Ordering::Acquire);

        tracing::info!("processing links task...");

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

            // send its file name and thumb if exists so that information of uploading successful can be showed
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

            // all task should be new, so this should always be 0
            let current_length = upload_session_meta
                .next_expected_ranges
                .first()
                .map_or(0, |range| range.start);

            let chat_bot_hex = message.chat().pack().to_hex();
            let chat_user_hex = chat_user.pack().to_hex();
            let chat_origin_hex = message_origin.chat().pack().to_hex();

            let _aborters = state.task_session.aborters.lock().await;

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
                    chat_origin_hex: Some(chat_origin_hex),
                    message_id,
                    message_id_forward: None,
                    message_id_origin: Some(message_origin.id()),
                    auto_delete,
                })
                .await?;

            tracing::info!(
                "inserted link task for links: {} size: {}",
                filename,
                total_length
            );
        }

        if should_delete {
            // command message is useless now, delete it
            telegram_user
                .get_message(chat_user, message.id())
                .await?
                .delete()
                .await?;
        }
    } else {
        return Err(Error::new(format_unknown_command_help(PATTERN)).parser_type(ParserType::Html));
    }

    Ok(())
}
