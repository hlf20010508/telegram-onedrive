/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    message::{ChatEntity, TelegramMessage},
    state::AppState,
};
use anyhow::{Context, Result};
use proc_macros::{check_in_group, check_senders, check_tg_login};

pub const PATTERN: &str = "/clear";

#[check_tg_login]
#[check_senders]
#[check_in_group]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let task_session = &state.task_session;

    task_session.clear().await?;

    let chat = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    loop {
        let mut messages = telegram_user.iter_messages(&chat).limit(100);

        let mut message_ids = Vec::new();

        while let Some(message) = messages
            .next()
            .await
            .context("failed to get next message")?
        {
            let id = message.id();
            // id 1 message is a service message that always exists when the group was created and it cannot be deleted
            if id != 1 {
                message_ids.push(id);
            }
        }

        if message_ids.is_empty() {
            break;
        }

        telegram_user.delete_messages(&chat, &message_ids).await?;
    }

    Ok(())
}
