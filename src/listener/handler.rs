/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{EventType, Events};
use crate::{message::TelegramMessage, state::AppState};
use anyhow::Result;
use grammers_client::types::Media;

pub struct Handler<'h> {
    pub events: &'h Events,
    pub state: &'h AppState,
}

impl<'h> Handler<'h> {
    pub fn new(events: &'h Events, state: &'h AppState) -> Self {
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

        self.trigger(EventType::Batch, message).await
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
