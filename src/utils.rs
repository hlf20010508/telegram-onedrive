/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use chrono::Utc;

pub fn get_current_timestamp() -> i64 {
    Utc::now().timestamp()
}
