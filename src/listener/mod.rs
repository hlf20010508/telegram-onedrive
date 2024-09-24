/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod events;
mod handler;

use grammers_client::Update;
use proc_macros::{add_context, add_trace};
use std::rc::Rc;
use std::sync::Arc;

pub use events::{EventType, HashMapExt};

use events::Events;
use handler::Handler;

use crate::env::BYPASS_PREFIX;
use crate::error::{Result, ResultUnwrapExt};
use crate::message::TelegramMessage;
use crate::state::{AppState, State};
use crate::tasker::Tasker;
use crate::trace::indenter;

pub struct Listener {
    pub events: Rc<Events>,
    pub state: AppState,
}

impl Listener {
    pub async fn new(events: Events) -> Self {
        let events = Rc::new(events);
        let state = Arc::new(State::new().await);

        Self { events, state }
    }

    pub async fn run(self) {
        tracing::info!("listener started");

        let tasker = Tasker::new(self.state.clone());
        tokio::spawn(async move {
            indenter::set_file_indenter(indenter::Coroutine::Task, async {
                tasker.run().await;
            })
            .await;
        });

        let telegram_user = self.state.telegram_user.raw().clone();
        tokio::spawn(async move {
            telegram_user.run_until_disconnected().await.unwrap();
        });

        loop {
            if let Err(e) = self.handle_message().await {
                e.trace();
            }
        }
    }

    #[add_context]
    #[add_trace]
    async fn handle_message(&self) -> Result<()> {
        let handler = Handler::new(self.events.clone(), self.state.clone());

        let client = &handler.state.telegram_bot;

        let update = client.next_update().await?;

        if let Update::NewMessage(message_raw) = update {
            if !message_raw.outgoing() && !message_raw.text().starts_with(BYPASS_PREFIX) {
                let message = TelegramMessage::new(client.clone(), message_raw);

                if let Err(e) = handler.handle_message(message.clone()).await {
                    e.send(message).await.unwrap_both().trace();
                }
            }
        }

        Ok(())
    }
}
