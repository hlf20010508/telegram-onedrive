/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;
use proc_macros::{add_context, add_trace};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;

use crate::client::TelegramClient;
use crate::env::LOGS_PATH;
use crate::error::{Error, Result};
use crate::message::TelegramMessage;

#[add_context]
#[add_trace]
pub async fn send_log_zip(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    const ZIP_PATH: &str = "./logs.zip";

    zip_dir(LOGS_PATH, ZIP_PATH)?;

    let file = telegram_bot.upload_file(ZIP_PATH).await?;

    message.respond(InputMessage::default().file(file)).await?;

    std::fs::remove_file(ZIP_PATH).map_err(|e| Error::new_sys_io(e, "failed to remove file"))?;

    Ok(())
}

#[add_context]
#[add_trace]
fn zip_dir<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let zip_file = std::fs::File::create(&output_path)
        .map_err(|e| Error::new_sys_io(e, "failed to create file"))?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default();

    let mut buffer = Vec::new();

    let entries =
        std::fs::read_dir(&input_path).map_err(|e| Error::new_sys_io(e, "failed to read dir"))?;

    for entry in entries {
        let entry = entry.map_err(|e| Error::new_sys_io(e, "failed to read dir entry"))?;

        let path = entry.path();
        let name = path
            .strip_prefix(&input_path)
            .map_err(|_| Error::new("failed to strip prefix"))?;

        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)
                .map_err(|e| Error::new_zip(e, "failed to start zip file"))?;

            let mut file = std::fs::File::open(path)
                .map_err(|e| Error::new_sys_io(e, "failed to open file"))?;

            file.read_to_end(&mut buffer)
                .map_err(|e| Error::new_sys_io(e, "failed to read file"))?;
            zip.write_all(&buffer)
                .map_err(|e| Error::new_sys_io(e, "failed to write file"))?;
            buffer.clear();
        } else {
            zip.add_directory(name.to_string_lossy(), options)
                .map_err(|e| Error::new_zip(e, "failed to add directory to zip"))?;
        }
    }

    zip.finish()
        .map_err(|e| Error::new_zip(e, "failed to finish zip"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zip_dir() {
        zip_dir(LOGS_PATH, "./logs.zip").unwrap();
    }
}
