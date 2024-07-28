/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::Media;
use proc_macros::add_trace;
use std::rc::Rc;

use super::{EventType, Events};
use crate::error::Result;
use crate::message::TelegramMessage;
use crate::state::AppState;

pub struct Handler {
    pub events: Rc<Events>,
    pub state: AppState,
}

impl Handler {
    #[add_trace]
    pub fn new(events: Rc<Events>, state: AppState) -> Self {
        Self { events, state }
    }

    #[add_trace(context)]
    pub async fn handle_message(&self, message: TelegramMessage) -> Result<()> {
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

    #[add_trace(context)]
    pub async fn handle_command(&self, message: TelegramMessage) -> Result<()> {
        let text = message.text();

        for event in self.get_event_names() {
            if text.starts_with(event.to_str()) {
                self.trigger(event, message).await?;
                break;
            }
        }

        Ok(())
    }

    #[add_trace(context)]
    pub async fn handle_text(&self, message: TelegramMessage) -> Result<()> {
        self.trigger(EventType::Text, message).await
    }

    #[add_trace(context)]
    pub async fn handle_media(&self, message: TelegramMessage) -> Result<()> {
        self.trigger(EventType::Media, message).await
    }

    #[add_trace(context)]
    async fn trigger(&self, event_name: EventType, message: TelegramMessage) -> Result<()> {
        if let Some(callback) = self.events.get(event_name.to_str()) {
            callback(message, self.state.clone()).await?;
        }

        Ok(())
    }

    #[add_trace]
    pub fn get_event_names(&self) -> Vec<EventType> {
        self.events.keys().map(EventType::from).collect()
    }
}
