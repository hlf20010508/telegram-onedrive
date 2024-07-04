/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Message;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::state::AppState;
use crate::{check_in_group, check_senders, check_tg_login};

pub const PATTERN: &str = "/clear";

pub async fn handler(message: Arc<Message>, state: AppState) -> Result<()> {
    check_in_group!(message);
    check_senders!(message, state);
    check_tg_login!(message, state);

    let telegram_user = &state.telegram_user;

    let mut dialogs = telegram_user.client.iter_dialogs();

    let chat = {
        let bot_chat = message.chat();

        loop {
            if let Some(dialog) = dialogs
                .next()
                .await
                .map_err(|e| Error::context(e, "failed to get dialog"))?
            {
                let user_chat = dialog.chat();
                if user_chat.id() == bot_chat.id() {
                    break user_chat.to_owned();
                }
            }
        }
    };

    loop {
        let mut messages = telegram_user.client.iter_messages(&chat).limit(100);

        let mut message_ids = Vec::new();

        while let Some(message) = messages
            .next()
            .await
            .map_err(|e| Error::context(e, "failed to get next message"))?
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

        telegram_user
            .client
            .delete_messages(&chat, &message_ids)
            .await
            .map_err(|e| Error::context(e, "failed to delete messages"))?;
    }

    Ok(())
}