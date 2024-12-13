/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::ResultExt;

use super::{
    utils::get_env_value,
    var::{RECONNECTION_POLICY, TG_USER_SESSION_PATH},
};

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
        let api_id = get_env_value("tg_api_id").unwrap_or_trace();
        let api_hash = get_env_value("tg_api_hash").unwrap_or_trace();
        let users = Self::parse_users();
        let phone_number = get_env_value("tg_user_phone").unwrap_or_trace();
        let password = get_env_value("tg_user_password").ok();
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
        let arg: Option<String> = get_env_value("tg_user_name").ok();

        let users = arg.map_or_else(Vec::new, |user_names| {
            let users = user_names.split(',').map(|s| s.to_string()).collect();

            users
        });

        users
    }
}
