/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{Context, Result};
use std::{env, str::FromStr};

pub fn get_env_value<T>(name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value_s = env::var(name)
        .context("failed to get env value")
        .context(name.to_string())?;

    let value = value_s
        .parse::<T>()
        .context("failed to parse env value")
        .context(format!("{}={}", name, value_s))?;

    Ok(value)
}

pub fn get_env_value_option<T>(name: &str, default: T) -> T
where
    T: FromStr,
{
    match env::var(name) {
        Ok(value_s) => value_s.parse::<T>().unwrap_or(default),
        Err(_) => default,
    }
}

// to be compatible with the python version
pub fn get_env_value_option_legacy<T>(names: &[&str], default: T) -> T
where
    T: FromStr,
{
    for name in names {
        if let Ok(value_s) = env::var(name) {
            return value_s.parse::<T>().unwrap_or(default);
        }
    }

    default
}
