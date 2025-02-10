/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2025 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tokio::sync::{Mutex, RwLock};

// progress_map: task id -> progress item
// chat_bot_hex_record: chat bot hex -> chat record
pub struct Progress {
    progress_map: RwLock<HashMap<i64, ProgressItem>>,
    chat_bot_hex_record: Mutex<HashMap<String, ChatRecord>>,
}

impl Progress {
    pub fn new() -> Self {
        Self {
            progress_map: RwLock::new(HashMap::new()),
            chat_bot_hex_record: Mutex::new(HashMap::new()),
        }
    }

    pub async fn insert(
        &self,
        task_id: i64,
        total_length: u64,
        chat_bot_hex: &str,
        chat_user_hex: &str,
        message_id: i32,
        filename: &str,
    ) {
        self.progress_map.write().await.insert(
            task_id,
            ProgressItem::new(total_length, chat_bot_hex, message_id, filename),
        );

        let mut chat_bot_hex_record = self.chat_bot_hex_record.lock().await;

        if let Some(record) = chat_bot_hex_record.get_mut(chat_bot_hex) {
            record.message_number += 1;
        } else {
            chat_bot_hex_record.insert(
                chat_bot_hex.to_string(),
                ChatRecord {
                    message_number: 1,
                    progress_message_id: None,
                    current_response: String::new(),
                    last_response: String::new(),
                    chat_user_hex: chat_user_hex.to_string(),
                },
            );
        }
    }

    pub async fn remove(&self, task_id: i64) {
        let mut progress_map = self.progress_map.write().await;

        if let Some(item) = progress_map.remove(&task_id) {
            let mut chat_bot_hex_record = self.chat_bot_hex_record.lock().await;

            if let Some(record) = chat_bot_hex_record.get_mut(&item.chat_bot_hex) {
                record.message_number -= 1;
            }
        }
    }

    pub async fn set_current_length(&self, task_id: i64, current_length: u64) -> Result<()> {
        let mut progress_map = self.progress_map.write().await;

        if let Some(item) = progress_map.get_mut(&task_id) {
            item.current_length = current_length;
            Ok(())
        } else {
            Err(anyhow!("task_id not found"))
        }
    }

    pub(super) async fn iter_item(&self) -> Vec<ProgressItem> {
        self.progress_map.read().await.values().cloned().collect()
    }
}

impl Progress {
    pub(super) async fn add_to_current_response(&self, chat_bot_hex: &str, s: &str) {
        let mut chat_bot_hex_record = self.chat_bot_hex_record.lock().await;

        if let Some(record) = chat_bot_hex_record.get_mut(chat_bot_hex) {
            if record.current_response.is_empty() {
                record.current_response = String::from("Progress:\n");
            }

            record.current_response += s;
        }
    }

    pub(super) async fn iter_record(&self) -> Vec<(String, ChatRecord)> {
        self.chat_bot_hex_record
            .lock()
            .await
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub(super) async fn update_progress_message_id(
        &self,
        chat_bot_hex: &str,
        progress_message_id: i32,
    ) {
        let mut chat_bot_hex_record = self.chat_bot_hex_record.lock().await;

        if let Some(record) = chat_bot_hex_record.get_mut(chat_bot_hex) {
            record.progress_message_id = Some(progress_message_id);
        }
    }

    pub(super) async fn update_last_response(&self, chat_bot_hex: &str, current_response: &str) {
        let mut chat_bot_hex_record = self.chat_bot_hex_record.lock().await;

        if let Some(record) = chat_bot_hex_record.get_mut(chat_bot_hex) {
            record.last_response = current_response.to_string();
            record.current_response = String::new();
        }
    }

    pub(super) async fn remove_record(&self, chat_bot_hex: &str) {
        self.chat_bot_hex_record.lock().await.remove(chat_bot_hex);
    }
}

#[derive(Debug, Clone)]
pub(super) struct ProgressItem {
    pub current_length: u64,
    pub total_length: u64,
    pub chat_bot_hex: String,
    pub message_id: i32,
    pub filename: String,
}

impl ProgressItem {
    fn new(total_length: u64, chat_bot_hex: &str, message_id: i32, filename: &str) -> Self {
        Self {
            current_length: 0,
            total_length,
            chat_bot_hex: chat_bot_hex.to_string(),
            message_id,
            filename: filename.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct ChatRecord {
    pub message_number: u8,
    pub progress_message_id: Option<i32>,
    pub current_response: String,
    pub last_response: String,
    pub chat_user_hex: String,
}
