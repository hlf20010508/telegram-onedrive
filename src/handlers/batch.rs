/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{auto_delete, dir, link, links, url};
use crate::{
    error::Result,
    message::{ChatEntity, TelegramMessage},
    state::AppState,
};
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    for (i, line) in message.text().split('\n').enumerate() {
        let detail = format!("line {}: {}", i + 1, line);

        if line.starts_with(auto_delete::PATTERN) {
            auto_delete::handler(message.clone(), state.clone())
                .await
                .details(detail)?;
        } else if line.starts_with(dir::PATTERN) {
            dir::handle_dir(message.clone(), line, state.clone())
                .await
                .details(detail)?;
        } else if line.starts_with(links::PATTERN) {
            links::handle_links(message.clone(), line, state.clone(), false)
                .await
                .details(detail)?;
        } else if line.starts_with(url::PATTERN) {
            url::handler_url(message.clone(), line, state.clone(), true)
                .await
                .details(detail)?;
        } else {
            link::handle_link(message.clone(), line, state.clone(), false)
                .await
                .details(detail)?;
        }
    }

    let telegram_user = &state.telegram_user;

    let chat_user = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    telegram_user
        .get_message(chat_user, message.id())
        .await?
        .delete()
        .await?;

    Ok(())
}
