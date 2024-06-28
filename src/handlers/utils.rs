/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::fmt::Display;

pub fn cmd_parser<T>(cmd: T) -> Vec<String>
where
    T: Display,
{
    cmd.to_string().split(" ").map(|s| s.to_string()).collect()
}
