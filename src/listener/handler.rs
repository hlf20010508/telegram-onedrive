/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::{Media, Message};
use std::sync::Arc;

use super::{EventType, Events};
use crate::error::Result;
use crate::state::AppState;

pub struct Handler {
    pub events: Arc<Events>,
    pub state: AppState,
}

impl Handler {
    pub fn new(events: Arc<Events>, state: AppState) -> Self {
        Self { events, state }
    }

    pub async fn handle_message(&self, message: Arc<Message>) -> Result<()> {
        match message.media() {
            Some(media) => match media {
                Media::Photo(_) | Media::Document(_) | Media::Sticker(_) => {
                    self.handle_media(message).await?
                }
                _ => {}
            },
            None => {
                let text = message.text();

                if !text.is_empty() {
                    if text.starts_with('/') {
                        self.handle_command(message).await?;
                    } else {
                        self.handle_text(message).await?;
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn handle_command(&self, message: Arc<Message>) -> Result<()> {
        let text = message.text();

        for event in self.get_event_names() {
            if text.starts_with(event.to_str()) {
                self.trigger(event, message).await?;
                break;
            }
        }

        Ok(())
    }

    pub async fn handle_text(&self, message: Arc<Message>) -> Result<()> {
        self.trigger(EventType::Text, message).await
    }

    pub async fn handle_media(&self, message: Arc<Message>) -> Result<()> {
        self.trigger(EventType::Media, message).await
    }

    async fn trigger(&self, event_name: EventType, message: Arc<Message>) -> Result<()> {
        if let Some(callback) = self.events.get(event_name.to_str()) {
            callback(message, self.state.clone()).await?;
        }

        Ok(())
    }

    pub fn get_event_names(&self) -> Vec<EventType> {
        self.events
            .keys()
            .map(|name| EventType::from(name))
            .collect()
    }
}
