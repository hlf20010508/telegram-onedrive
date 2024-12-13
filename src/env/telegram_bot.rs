/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::ResultExt;

use super::{
    utils::get_env_value,
    var::{RECONNECTION_POLICY, TG_BOT_SESSION_PATH},
};

pub struct TelegramBotEnv {
    pub api_id: i32,
    pub api_hash: String,
    pub token: String,
    pub session_path: String,
    pub params: grammers_client::InitParams,
}

impl TelegramBotEnv {
    pub fn new() -> Self {
        let api_id = get_env_value("tg_api_id").unwrap_or_trace();
        let api_hash = get_env_value("tg_api_hash").unwrap_or_trace();
        let token = get_env_value("tg_bot_token").unwrap_or_trace();
        let session_path = TG_BOT_SESSION_PATH.to_string();
        let params = grammers_client::InitParams {
            reconnection_policy: &RECONNECTION_POLICY,
            ..Default::default()
        };

        Self {
            api_id,
            api_hash,
            token,
            session_path,
            params,
        }
    }
}
