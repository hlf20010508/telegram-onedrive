/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod events;
mod handler;

use grammers_client::Update;
use std::sync::Arc;

pub use events::{EventType, HashMapExt};

use events::Events;
use handler::Handler;

use crate::env::BYPASS_PREFIX;
use crate::error::{Error, Result, ResultExt};
use crate::state::{AppState, State};
use crate::tasker::Tasker;

pub struct Listener {
    pub events: Arc<Events>,
    pub state: AppState,
}

impl Listener {
    pub async fn new(events: Events) -> Self {
        let events = Arc::new(events);
        let state = Arc::new(State::new().await);

        Self { events, state }
    }

    pub async fn run(self) {
        tracing::debug!("listener started");

        let tasker = Tasker::new(self.state.clone()).await.unwrap();
        tokio::spawn(async move {
            tasker.run().await;
        });

        loop {
            if let Err(e) = self.handle_message().await {
                e.trace();
            }
        }
    }

    async fn handle_message(&self) -> Result<()> {
        let handler = Handler::new(self.events.clone(), self.state.clone());

        let update = handler
            .state
            .telegram_bot
            .client
            .next_update()
            .await
            .map_err(|e| Error::context(e, "Failed to get next update"))?;

        if let Some(Update::NewMessage(message)) = update {
            if !message.outgoing() && !message.text().starts_with(BYPASS_PREFIX) {
                let message = Arc::new(message);

                if let Err(e) = handler.handle_message(message.clone()).await {
                    e.send(message).await.unwrap_both().trace()
                }
            }
        }

        Ok(())
    }
}
