/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::{
    utils::get_arg_value,
    var::{RECONNECTION_POLICY, TG_USER_SESSION_PATH},
};
use crate::error::ResultExt;

pub struct TelegramUserEnv {
    pub api_id: i32,
    pub api_hash: String,
    pub users: Vec<String>,
    pub phone_number: String,
    pub password: Option<String>,
    pub session_path: String,
    pub params: grammers_client::InitParams,
}

impl TelegramUserEnv {
    pub fn new() -> Self {
        let api_id = get_arg_value("--tg-api-id").unwrap_or_trace();
        let api_hash = get_arg_value("--tg-api-hash").unwrap_or_trace();
        let users = Self::parse_users();
        let phone_number = get_arg_value("--tg-user-phone").unwrap_or_trace();
        let password = get_arg_value("--tg-user-password").ok();
        let session_path = TG_USER_SESSION_PATH.to_string();
        let params = grammers_client::InitParams {
            reconnection_policy: &RECONNECTION_POLICY,
            ..Default::default()
        };

        Self {
            api_id,
            api_hash,
            users,
            phone_number,
            password,
            session_path,
            params,
        }
    }

    fn parse_users() -> Vec<String> {
        let arg: Option<String> = get_arg_value("--tg-user-name").ok();

        let users = if let Some(user_names) = arg {
            let users = user_names.split(',').map(|s| s.to_string()).collect();

            users
        } else {
            Vec::new()
        };

        users
    }
}
