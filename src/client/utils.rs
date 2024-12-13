/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{Context, Result};
use grammers_client::types::PackedChat;

pub fn chat_from_hex(chat_hex: &str) -> Result<PackedChat> {
    PackedChat::from_hex(chat_hex).context("failed to parse chat hex to packed chat")
}
