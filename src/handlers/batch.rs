/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    error::{Error, Result, ResultUnwrapExt},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
};
use grammers_client::types::Downloadable;
use proc_macros::{add_context, add_trace};

#[add_context]
#[add_trace]
pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = state.telegram_user.clone();

    let chat_user = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_user
        .media()
        .ok_or_else(|| Error::new("message does not contain any media"))?;

    let downloadable = Downloadable::Media(media);
    let mut download = telegram_user.iter_download(&downloadable);
    let mut batch_bytes = Vec::new();
    while let Some(chunk) = download
        .next()
        .await
        .map_err(|e| Error::new("failed to get next chunk from tg file downloader").raw(e))?
    {
        batch_bytes.extend(chunk);
    }

    let batch =
        String::from_utf8(batch_bytes).map_err(|e| Error::new("failed to parse batch").raw(e))?;

    tokio::spawn(async move {
        let batch = batch.trim();

        for (i, line) in batch.split('\n').enumerate() {
            let detail = format!("line {}: {}", i + 1, line);

            if let Err(e) = telegram_user
                .send_message(&chat_user, line)
                .await
                .context("failed to send command in batch")
                .details(detail)
            {
                e.send(message.clone()).await.unwrap_both().trace();
            }
        }

        if let Err(e) = message_user.delete().await {
            e.send(message.clone()).await.unwrap_both().trace();
        }
    });

    Ok(())
}
