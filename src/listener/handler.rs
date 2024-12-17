/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{EventType, Events};
use crate::{
    error::{ErrorExt, ResultUnwrapExt},
    message::{ChatEntity, TelegramMessage},
    state::AppState,
};
use anyhow::{anyhow, Context, Result};
use grammers_client::types::{Downloadable, Media};

pub struct Handler {
    pub events: Events,
    pub state: AppState,
}

impl Handler {
    pub fn new(events: Events, state: AppState) -> Self {
        Self { events, state }
    }

    pub async fn handle_message(&self, message: TelegramMessage) -> Result<()> {
        match message.media() {
            Some(media) => match media {
                Media::Document(document) if document.name().to_lowercase().ends_with(".t2o") => {
                    self.handle_batch(message).await?;
                }
                Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => {
                    self.handle_media(message).await?;
                }
                // sending a task with a link may cause the text being wrapped as a web page
                Media::WebPage(_) => self.handle_text(message).await?,
                _ => tracing::debug!("unsupported media type when handle message"),
            },
            None => self.handle_text(message).await?,
        }

        Ok(())
    }

    async fn handle_command(&self, message: TelegramMessage) -> Result<()> {
        let text = message.text();

        for event in self.get_event_names() {
            if text.starts_with(event.to_str()) {
                tracing::info!("handle command {}", event);

                self.trigger(event, message).await?;
                break;
            }
        }

        Ok(())
    }

    async fn handle_text(&self, message: TelegramMessage) -> Result<()> {
        let text = message.text().trim();

        if !text.is_empty() {
            if text.starts_with('/') {
                self.handle_command(message.clone()).await?;
            } else {
                tracing::info!("handle text");

                self.trigger(EventType::Text, message.clone()).await?;
            }
        }

        Ok(())
    }

    async fn handle_media(&self, message: TelegramMessage) -> Result<()> {
        tracing::info!("handle media");

        self.trigger(EventType::Media, message).await
    }

    async fn handle_batch(&self, message: TelegramMessage) -> Result<()> {
        tracing::info!("handle batch");

        let telegram_user = self.state.telegram_user.clone();

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
        let handler = Self::new(self.events.clone(), self.state.clone());
        tokio::spawn(async move {
            let batch = batch.trim();

            for (i, line) in batch.split('\n').enumerate() {
                let detail = format!("line {}: {}", i + 1, line);

                let mut message_clone = message.clone();
                message_clone.override_text(line.to_string());

                if let Err(e) = handler
                    .handle_text(message_clone)
                    .await
                    .context("failed to send command in batch")
                    .context(detail)
                {
                    e.send(message.clone()).await.unwrap_both().trace();

                    continue;
                }
            }
        });

        Ok(())
    }

    async fn trigger(&self, event_name: EventType, message: TelegramMessage) -> Result<()> {
        if let Some(callback) = self.events.get(event_name.to_str()) {
            callback(message, self.state.clone()).await?;
        }

        Ok(())
    }

    fn get_event_names(&self) -> Vec<EventType> {
        self.events.keys().map(EventType::from).collect()
    }
}
