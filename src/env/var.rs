/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::FixedReconnect;
use std::time::Duration;

pub const LOGS_PATH: &str = "./logs";

pub const SESSION_DIR: &str = "./session";
pub const TG_BOT_SESSION_PATH: &str = "./session/tg-bot.session";
pub const TG_USER_SESSION_PATH: &str = "./session/tg-user.session";
pub const OD_SESSION_PATH: &str = "./session/od.session";
pub const TASKER_SESSION_PATH: &str = "./session/tasker.session";

pub const RECONNECTION_POLICY: FixedReconnect = FixedReconnect {
    attempts: 5,
    delay: Duration::from_secs(1),
};
