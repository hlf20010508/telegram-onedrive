/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::utils::{get_arg_value, get_arg_value_option};
use super::var::OD_SESSION_PATH;

pub struct OneDriveEnv {
    pub client_id: String,
    pub client_secret: String,
    pub root_path: String,
    pub session_path: String,
}

impl OneDriveEnv {
    pub fn new() -> Self {
        let client_id = get_arg_value("--od-client-id").unwrap();
        let client_secret = get_arg_value("--od-client-secret").unwrap();
        let root_path = get_arg_value_option("--od-root-path", "/".to_string());
        let session_path = OD_SESSION_PATH.to_string();

        Self {
            client_id,
            client_secret,
            root_path,
            session_path,
        }
    }
}
