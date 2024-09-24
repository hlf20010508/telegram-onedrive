/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use proc_macros::{add_context, add_trace, check_in_group, check_senders};

use crate::error::{Error, Result};
use crate::message::{ChatEntity, TelegramMessage};
use crate::state::AppState;

pub const PATTERN: &str = "/testEdit";

#[check_senders]
#[check_in_group]
#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let response = "test response";
    let respond_message = message.respond(response).await.details(response)?;

    let telegram_bot = &state.telegram_bot;
    let telegram_user = &state.telegram_user;

    for i in 0..20 {
        let latest_message = telegram_user
            .iter_messages(
                telegram_user
                    .get_chat(&ChatEntity::Chat(message.chat()))
                    .await?,
            )
            .limit(1)
            .next()
            .await
            .map_err(|e| {
                Error::new_telegram_invocation(e, "failed to iter messages for latest message")
            })?;

        telegram_bot
            .edit_message(
                message.chat(),
                respond_message.id(),
                format!("test edit {}", i),
            )
            .await?;
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
