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

use std::fs;

pub use onedrive::OneDriveEnv;
pub use telegram_bot::TelegramBotEnv;
pub use telegram_user::TelegramUserEnv;
pub use var::{BYPASS_PREFIX, LOGS_PATH, WORKER_NUM};

use utils::{args_contains, get_arg_value, get_arg_value_option};
use var::SESSION_DIR;

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
}

impl Env {
    pub fn new() -> Self {
        Self::init();

        let telegram_bot = TelegramBotEnv::new();
        let telegram_user = TelegramUserEnv::new();
        let onedrive = OneDriveEnv::new();
        let trace_level = get_arg_value_option("--trace-level", "info".to_string());
        let port = get_arg_value_option("--port", 8080);
        let server_uri = get_arg_value("--server-uri").unwrap();
        let use_reverse_proxy = args_contains("--reverse-proxy");
        let should_auto_delete = args_contains("--auto-delete");
        let tasker_session_path = var::TASKER_SESSION_PATH.to_string();

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
        }
    }

    fn init() {
        fs::create_dir_all(SESSION_DIR).unwrap();
    }
}
