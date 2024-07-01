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

use crate::error::Error;
use crate::state::{AppState, State};

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
        loop {
            let handler = Handler::new(self.events.clone(), self.state.clone());

            let result = match handler.state.telegram_bot.client.next_update().await {
                Ok(update) => match update {
                    Some(update) => match update {
                        Update::NewMessage(message) if !message.outgoing() => {
                            let message = Arc::new(message);

                            match handler.handle_message(message.clone()).await {
                                Ok(_) => Ok(()),
                                Err(e) => Err(e.send(message).await.unwrap()),
                            }
                        }
                        _ => Ok(()),
                    },
                    None => Ok(()),
                },
                Err(e) => Err(Error::context(e, "Failed to get next update")),
            };

            if let Err(e) = result {
                e.trace();
            }
        }
    }
}
