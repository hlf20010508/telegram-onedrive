/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::{
    env::LOGS_PATH,
    error::{Error, ResultExt},
};
use chrono::{Local, NaiveDate};
use regex::Regex;
use std::{
    fs,
    path::Path,
    thread::{sleep, spawn},
    time::Duration,
};

pub fn run() {
    // remove logs older than 7 days
    spawn(|| loop {
        let pattern = r"(\d{4}-\d{2}-\d{2})\.\w+\.log";
        let re = Regex::new(pattern)
            .map_err(|e| Error::new("invalid regex pattern").raw(e).details(pattern))
            .unwrap_or_trace();

        let cutoff_date = Local::now().naive_local().date() - chrono::Duration::days(7);

        if Path::new(LOGS_PATH).exists() {
            for entry in fs::read_dir(LOGS_PATH)
                .map_err(|e| Error::new("failed to read logs dir").raw(e))
                .unwrap_or_trace()
            {
                let entry = entry
                    .map_err(|e| Error::new("failed to visit log file entry").raw(e))
                    .unwrap_or_trace();
                let path = entry.path();

                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if let Some(caps) = re.captures(filename) {
                        if let Some(date_str) = caps.get(1).map(|m| m.as_str()) {
                            if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if file_date < cutoff_date {
                                    fs::remove_file(&path)
                                        .map_err(|e| {
                                            Error::new("failed to remove file")
                                                .raw(e)
                                                .details(path.to_string_lossy())
                                        })
                                        .trace();
                                }
                            }
                        }
                    }
                }
            }
        }

        // exec every hour
        sleep(Duration::from_secs(60 * 60));
    });
}
