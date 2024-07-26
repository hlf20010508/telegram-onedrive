/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::types::PackedChat;

use crate::error::{Error, Result};

pub fn chat_from_hex(chat_hex: &str) -> Result<PackedChat> {
    PackedChat::from_hex(chat_hex)
        .map_err(|e| Error::new_telegram_packed_chat("failed to parse chat hex to packed chat", e))
}
