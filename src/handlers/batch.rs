/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    error::{ErrorExt, ResultUnwrapExt},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
};
use anyhow::{anyhow, Context, Result};
use grammers_client::types::Downloadable;

pub async fn handler(message: TelegramMessage, state: AppState) -> Result<()> {
    let telegram_user = state.telegram_user.clone();

    let chat_user = telegram_user
        .get_chat(&ChatEntity::from(message.chat()))
        .await?;

    let message_user = telegram_user.get_message(&chat_user, message.id()).await?;

    let media = message_user
        .media()
        .ok_or_else(|| anyhow!("message does not contain any media"))?;

    let downloadable = Downloadable::Media(media);
    let mut download = telegram_user.iter_download(&downloadable);
    let mut batch_bytes = Vec::new();
    while let Some(chunk) = download
        .next()
        .await
        .context("failed to get next chunk from tg file downloader")?
    {
        batch_bytes.extend(chunk);
    }

    let batch = String::from_utf8(batch_bytes).context("failed to parse batch")?;

    tokio::spawn(async move {
        let batch = batch.trim();

        for (i, line) in batch.split('\n').enumerate() {
            let detail = format!("line {}: {}", i + 1, line);

            if let Err(e) = telegram_user
                .send_message(&chat_user, line)
                .await
                .context("failed to send command in batch")
                .context(detail)
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
