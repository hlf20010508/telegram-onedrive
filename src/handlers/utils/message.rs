/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    client::TelegramClient,
    error::{Error, Result},
    message::{ChatEntity, MessageInfo, TelegramMessage},
};
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub fn get_message_info(link: &str) -> Result<MessageInfo> {
    let (message_info, is_private) =
        if let Some(message_info) = link.strip_prefix("https://t.me/c/") {
            // link from private group
            (message_info, true)
        } else if let Some(message_info) = link.strip_prefix("https://t.me/") {
            // link from public group
            (message_info, false)
        } else {
            return Err(Error::new("not a message link"));
        };

    let message_info_vec = message_info
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if message_info_vec.len() != 2 {
        return Err(Error::new("message info doesn't contain 2 elements"));
    }

    let chat_entity = if is_private {
        let chat_id = message_info_vec[0]
            .parse::<i64>()
            .map_err(|e| Error::new("failed to parse chat id").raw(e))?;

        ChatEntity::from(chat_id)
    } else {
        let chat_name = message_info_vec[0].clone();

        ChatEntity::from(chat_name)
    };

    let message_id = message_info_vec[1]
        .parse()
        .map_err(|e| Error::new("failed to parse message id").raw(e))?;

    Ok(MessageInfo::new(chat_entity, message_id))
}

#[add_context]
#[add_trace]
pub async fn get_message_from_link(
    telegram_user: &TelegramClient,
    link: &str,
) -> Result<TelegramMessage> {
    let MessageInfo {
        chat_entity,
        id: message_id,
    } = get_message_info(link)?;

    let chat = telegram_user.get_chat(&chat_entity).await?;

    telegram_user.get_message(chat, message_id).await
}

pub fn get_message_link(chat_entity: &ChatEntity, id: i32) -> String {
    match chat_entity {
        ChatEntity::Chat(chat) => {
            if let Some(username) = chat.username() {
                // public group
                format!("https://t.me/{}/{}", username, id)
            } else {
                // private group
                format!("https://t.me/c/{}/{}", chat.id(), id)
            }
        }
        // private group
        ChatEntity::Id(chat_id) => format!("https://t.me/c/{}/{}", chat_id, id),
        // public group
        ChatEntity::Username(username) => format!("https://t.me/{}/{}", username, id),
    }
}
