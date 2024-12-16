/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod onedrive;
mod telegram_bot;
mod telegram_user;
mod utils;
mod var;

use anyhow::Context;
pub use onedrive::OneDriveEnv;
use std::{fs, sync::OnceLock};
pub use telegram_bot::TelegramBotEnv;
pub use telegram_user::TelegramUserEnv;
use utils::{get_env_value, get_env_value_option, get_env_value_option_legacy};
pub use var::LOGS_PATH;
use var::SESSION_DIR;

use crate::error::ResultExt;

pub static ENV: OnceLock<Env> = OnceLock::new();

pub struct Env {
    pub telegram_bot: TelegramBotEnv,
    pub telegram_user: TelegramUserEnv,
    pub onedrive: OneDriveEnv,
    pub trace_level: String,
    pub port: u16,
    pub server_uri: String,
    pub use_reverse_proxy: bool,
    pub should_auto_delete: bool,
    pub tasker_session_path: String,
    pub task_handler_num: u8,
}

impl Env {
    pub fn new() -> Self {
        Self::init();

        let telegram_bot = TelegramBotEnv::new();
        let telegram_user = TelegramUserEnv::new();
        let onedrive = OneDriveEnv::new();
        let trace_level = get_env_value_option("trace_level", "info".to_string());
        let port = get_env_value_option("port", 8080);
        let server_uri = get_env_value("server_uri").unwrap_or_trace();
        let use_reverse_proxy = get_env_value_option("reverse_proxy", false);
        let should_auto_delete =
            get_env_value_option_legacy(&["auto_delete", "delete_flag"], false);
        let tasker_session_path = var::TASKER_SESSION_PATH.to_string();
        let task_handler_num = get_env_value_option("worker_num", 5);

        Self {
            telegram_bot,
            telegram_user,
            onedrive,
            trace_level,
            port,
            server_uri,
            use_reverse_proxy,
            should_auto_delete,
            tasker_session_path,
            task_handler_num,
        }
    }

    fn init() {
        fs::create_dir_all(SESSION_DIR)
            .context("failed to create session dir")
            .unwrap_or_trace();
    }
}
