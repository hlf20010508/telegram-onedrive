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
pub use var::{LOG_PATH, WORKER_NUM};

use utils::{args_contains, get_arg_value, get_arg_value_option};
use var::SESSION_DIR;

pub struct Env {
    pub telegram_bot: TelegramBotEnv,
    pub telegram_user: TelegramUserEnv,
    pub onedrive: OneDriveEnv,
    pub port: u16,
    pub server_uri: String,
    pub use_reverse_proxy: bool,
    pub should_auto_delete: bool,
}

impl Env {
    pub fn new() -> Self {
        Self::init();

        let telegram_bot = TelegramBotEnv::new();
        let telegram_user = TelegramUserEnv::new();
        let onedrive = OneDriveEnv::new();
        let port = get_arg_value_option("--port", 8080);
        let server_uri = get_arg_value("--server-uri").unwrap();
        let use_reverse_proxy = args_contains("--reverse-proxy");
        let should_auto_delete = args_contains("--auto-delete");

        Env {
            telegram_bot,
            telegram_user,
            onedrive,
            port,
            server_uri,
            use_reverse_proxy,
            should_auto_delete,
        }
    }

    fn init() {
        fs::create_dir_all(SESSION_DIR).unwrap();
    }
}
