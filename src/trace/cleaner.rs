/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use chrono::{Local, NaiveDate};
use regex::Regex;
use std::fs;
use std::path::Path;
use std::thread::{sleep, spawn};
use std::time::Duration;

use crate::env::LOGS_PATH;

pub fn run() {
    // remove logs older than 7 days
    spawn(|| loop {
        let re = Regex::new(r"(\d{4}-\d{2}-\d{2})\.\w+\.log").unwrap();

        let cutoff_date = Local::now().naive_local().date() - chrono::Duration::days(7);

        if Path::new(LOGS_PATH).exists() {
            for entry in fs::read_dir(LOGS_PATH).unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();

                if let Some(filename) = path.file_name().and_then(|f| f.to_str()) {
                    if let Some(caps) = re.captures(filename) {
                        if let Some(date_str) = caps.get(1).map(|m| m.as_str()) {
                            if let Ok(file_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if file_date < cutoff_date {
                                    fs::remove_file(&path).unwrap();
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
