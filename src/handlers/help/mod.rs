/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod docs;

use grammers_client::types::Message;
use grammers_client::InputMessage;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::state::AppState;

pub const PATTERN: &str = "/help";

pub async fn handler(message: Arc<Message>, _state: AppState) -> Result<()> {
    message
        .respond(InputMessage::html(docs::GREETING))
        .await
        .map_err(|e| Error::context(e, "failed to respond /help"))?;

    Ok(())
}
