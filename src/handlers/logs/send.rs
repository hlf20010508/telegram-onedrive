/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use grammers_client::InputMessage;
use path_slash::PathExt;
use proc_macros::{add_context, add_trace};
use std::io::{Read, Write};
use std::path::Path;
use zip::write::{FileOptions, SimpleFileOptions};

use crate::client::TelegramClient;
use crate::env::LOGS_PATH;
use crate::error::{Error, Result};
use crate::message::TelegramMessage;

#[add_context]
#[add_trace]
pub async fn send_log_zip(telegram_bot: &TelegramClient, message: TelegramMessage) -> Result<()> {
    const ZIP_PATH: &str = "./logs.zip";

    zip_dir(LOGS_PATH, ZIP_PATH)?;

    let file = telegram_bot.upload_file(ZIP_PATH).await.context("logs")?;

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

    add_entry(input_path.as_ref(), input_path.as_ref(), &mut zip, options)?;

    zip.finish()
        .map_err(|e| Error::new_zip(e, "failed to finish zip"))?;

    Ok(())
}

fn add_entry(
    base_path: &Path,
    path: &Path,
    zip: &mut zip::ZipWriter<std::fs::File>,
    options: FileOptions<()>,
) -> Result<()> {
    let mut buffer = [0; 8];

    let name = path
        .strip_prefix(base_path)
        .map_err(|e| Error::new(format!("failed to strip prefix: {}", e)))?;

    // path seperator in zip must be slash /, not backslash \

    if path.is_file() {
        zip.start_file(name.to_slash_lossy(), options)
            .map_err(|e| Error::new_zip(e, "failed to start zip file"))?;

        let mut file =
            std::fs::File::open(path).map_err(|e| Error::new_sys_io(e, "failed to open file"))?;

        loop {
            let size = file
                .read(&mut buffer)
                .map_err(|e| Error::new_sys_io(e, "failed to read file"))?;

            if size == 0 {
                break;
            }

            zip.write_all(&buffer[..size])
                .map_err(|e| Error::new_sys_io(e, "failed to write file"))?;
        }
    } else if path.is_dir() {
        zip.add_directory(name.to_slash_lossy(), options)
            .map_err(|e| Error::new_zip(e, "failed to add directory to zip"))?;

        for entry in std::fs::read_dir(path)
            .map_err(|e| Error::new_sys_io(e, "failed to read dir").context("sub dir"))?
        {
            let entry = entry.map_err(|e| {
                Error::new_sys_io(e, "failed to read dir entry").context("sub dir entry")
            })?;

            add_entry(base_path, &entry.path(), zip, options)?;
        }
    }

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
