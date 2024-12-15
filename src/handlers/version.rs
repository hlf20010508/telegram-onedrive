/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{message::TelegramMessage, state::AppState};
use anyhow::Result;

pub const PATTERN: &str = "/version";

pub async fn handler(message: TelegramMessage, _state: AppState) -> Result<()> {
    let name = env!("CARGO_PKG_NAME");
    let version = env!("CARGO_PKG_VERSION");
    let authors = env!("CARGO_PKG_AUTHORS");

    let response = format!("{} v{} by {}", name, version, authors);

    message.respond(response).await?;

    Ok(())
}
