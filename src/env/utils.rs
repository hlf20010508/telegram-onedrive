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

pub fn get_arg_value<T>(arg_name: &'static str) -> Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    let value: T = args
        .value_from_str(arg_name)
        .map_err(|e| Error::new("failed to get arg").raw(e))?;

    Ok(value)
}

pub fn get_arg_value_option<T>(arg_name: &'static str, default: T) -> T
where
    T: FromStr,
    T::Err: Display,
{
    let mut args = Arguments::from_env();
    args.value_from_str(arg_name).unwrap_or(default)
}

pub fn args_contains(arg_name: &'static str) -> bool {
    let mut args = Arguments::from_env();
    args.contains(arg_name)
}
