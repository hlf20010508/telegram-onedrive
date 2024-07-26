/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use grammers_client::types::Message;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::state::AppState;

pub const PATTERN: &str = "/start";

pub async fn handler(message: Arc<Message>, _state: AppState) -> Result<()> {
    message
        .respond(docs::GREETING)
        .await
        .map_err(|e| Error::new_telegram_invocation(e, "failed to respond greating message"))?;

    Ok(())
}
