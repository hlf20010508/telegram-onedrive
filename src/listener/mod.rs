/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod events;
mod handler;

use crate::{
    error::{ErrorExt, ResultExt, ResultUnwrapExt},
    message::TelegramMessage,
    state::AppState,
};
use anyhow::{Ok, Result};
use events::Events;
pub use events::{EventType, HashMapExt};
use grammers_client::Update;
use handler::Handler;
use tokio::spawn;

pub struct Listener {
    pub events: Events,
    pub state: AppState,
}

impl Listener {
    pub fn new(events: Events, state: AppState) -> Self {
        Self { events, state }
    }

    pub async fn run(events: Events, state: AppState) {
        let listener = Self::new(events, state.clone());

        spawn(async move {
            tracing::info!("listener started");

            loop {
                listener.handle_message().await.trace();
            }
        })
        .await
        .unwrap();
    }

    async fn handle_message(&self) -> Result<()> {
        let client = &self.state.telegram_bot;

        let update = client.next_update().await?;
        if let Update::NewMessage(message_raw) = update {
            // bypass message that the bot sent itself
            if !message_raw.outgoing() {
                let message = TelegramMessage::new(client.clone(), message_raw);

                let handler = Handler::new(&self.events, self.state.clone());
                if let Err(e) = handler.handle_message(message.clone()).await {
                    e.send(message).await.unwrap_both().trace();
                }
            }
        }

        Ok(())
    }
}
