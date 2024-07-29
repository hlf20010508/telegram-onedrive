/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace, check_in_group, check_senders, check_tg_login};

use crate::error::{Error, Result};
use crate::message::TelegramMessage;
use crate::state::AppState;

pub const PATTERN: &str = "/clear";

#[check_tg_login]
#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = &state.telegram_user;
    let task_session = state.task_session.clone();

    task_session.clear().await?;

    let chat = telegram_user.get_chat(message).await?;

    loop {
        let mut messages = telegram_user.iter_messages(&chat).limit(100);

        let mut message_ids = Vec::new();

        while let Some(message) = messages
            .next()
            .await
            .map_err(|e| Error::new_telegram_invocation(e, "failed to get next message"))?
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
