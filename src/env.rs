/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use pico_args::Arguments;
use std::fmt::Display;
use std::str::FromStr;

use crate::error::{Error, Result};

const TG_BOT_SESSION_PATH: &str = "./session/tg-bot.session";

fn get_arg_value<T>(arg_name: &'static str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args
        .value_from_str(arg_name)
        .map_err(|e| Error::context(e, "failed to get arg"))?;

    Ok(value)
}

fn get_arg_value_option<T>(arg_name: &'static str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    args.value_from_str(arg_name).unwrap_or(default)
}

fn args_contains(arg_name: &'static str) -> bool {
    let mut args = Arguments::from_env();
    args.contains(arg_name)
}

pub struct Env {
    pub telegram_bot: TelegramBotEnv,
}

impl Env {
    pub fn new() -> Self {
        let telegram_bot = TelegramBotEnv::new();
        Env { telegram_bot }
    }
}

pub struct TelegramBotEnv {
    pub api_id: i32,
    pub api_hash: String,
    pub token: String,
    pub session_path: String,
    pub params: grammers_client::InitParams,
}

impl TelegramBotEnv {
    pub fn new() -> Self {
        let api_id = get_arg_value("--tg-api-id").unwrap();
        let api_hash = get_arg_value("--tg-api-hash").unwrap();
        let token = get_arg_value("--tg-bot-token").unwrap();
        let session_path = TG_BOT_SESSION_PATH.to_string();

        let params = grammers_client::InitParams {
            catch_up: true,
            ..Default::default()
        };

        TelegramBotEnv {
            api_id,
            api_hash,
            token,
            session_path,
            params,
        }
    }
}
