/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::{Error, RawError, Result};
use std::{env, str::FromStr};

pub fn get_env_value<T>(name: &str) -> Result<T>
where
    T: FromStr,
    T::Err: Into<RawError>,
{
    let value_s =
        env::var(name).map_err(|e| Error::new("failed to get env value").raw(e).details(name))?;

    let value = value_s.parse::<T>().map_err(|e| {
        Error::new("failed to parse env value")
            .raw(e)
            .details(format!("{}={}", name, value_s))
    })?;

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
