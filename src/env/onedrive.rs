/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::ResultExt;

use super::{
    utils::{get_env_value, get_env_value_option_legacy},
    var::OD_SESSION_PATH,
};

pub struct OneDriveEnv {
    pub client_id: String,
    pub client_secret: String,
    pub root_path: String,
    pub session_path: String,
}

impl OneDriveEnv {
    pub fn new() -> Self {
        let client_id = get_env_value("od_client_id").unwrap_or_trace();
        let client_secret = get_env_value("od_client_secret").unwrap_or_trace();
        let root_path =
            get_env_value_option_legacy(&["od_root_path", "remote_root_path"], "/".to_string());
        let session_path = OD_SESSION_PATH.to_string();

        Self {
            client_id,
            client_secret,
            root_path,
            session_path,
        }
    }
}
