/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    client::TelegramClient,
    message::{ChatEntity, MessageInfo, TelegramMessage},
};
use anyhow::{Context, Result, anyhow};

pub fn get_message_info(link: &str) -> Result<MessageInfo> {
    let (message_info, is_private) =
        if let Some(message_info) = link.strip_prefix("https://t.me/c/") {
            // link from private group
            (message_info, true)
        } else if let Some(message_info) = link.strip_prefix("https://t.me/") {
            // link from public group
            (message_info, false)
        } else {
            return Err(anyhow!("not a message link"));
        };

    let mut message_info_vec = message_info
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if message_info_vec.len() != 2 {
        if message_info_vec.len() == 3 {
            // link from a topic
            message_info_vec.remove(1);
        } else {
            return Err(anyhow!("message info doesn't contain 2 elements"));
        }
    }

    let chat_entity = if is_private {
        let chat_id = message_info_vec[0]
            .parse::<i64>()
            .context("failed to parse chat id")?;

        ChatEntity::from(chat_id)
    } else {
        let chat_name = message_info_vec[0].clone();

        ChatEntity::from(chat_name)
    };

    let message_id = message_info_vec[1]
        .parse()
        .context("failed to parse message id")?;

    Ok(MessageInfo::new(chat_entity, message_id))
}

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
        ChatEntity::Chat(chat) => chat.username().map_or_else(
            || format!("https://t.me/c/{}/{}", chat.id(), id),
            |username| format!("https://t.me/{}/{}", username, id),
        ),
        // private group
        ChatEntity::Id(chat_id) => format!("https://t.me/c/{}/{}", chat_id, id),
        // public group
        ChatEntity::Username(username) => format!("https://t.me/{}/{}", username, id),
    }
}

pub fn format_message_link(chat_id: i64, message_id: i32, filename: &str) -> String {
    format!(
        "<a href=\"https://t.me/c/{}/{}\">{}</a>",
        chat_id, message_id, filename
    )
}
