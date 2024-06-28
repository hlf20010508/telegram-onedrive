/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod telegram_bot;
mod telegram_user;
mod utils;
mod var;

pub use telegram_bot::TelegramBotEnv;
pub use telegram_user::TelegramUserEnv;
pub use var::LOG_PATH;

use utils::args_contains;

pub struct Env {
    pub telegram_bot: TelegramBotEnv,
    pub telegram_user: TelegramUserEnv,
    pub should_auto_delete: bool,
}

impl Env {
    pub fn new() -> Self {
        let telegram_bot = TelegramBotEnv::new();
        let telegram_user = TelegramUserEnv::new();
        let should_auto_delete = args_contains("--auto-delete");

        Env {
            telegram_bot,
            telegram_user,
            should_auto_delete,
        }
    }
}
