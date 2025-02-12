/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{Context, Result};
use async_zip::{
    tokio::write::ZipFileWriter, Compression, ZipDateTime, ZipDateTimeBuilder, ZipEntryBuilder,
};
use chrono::{DateTime, Datelike, Local, Timelike};
use futures::AsyncWriteExt as _;
use std::{path::Path, time::UNIX_EPOCH};
use tokio::{
    fs::{self, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

pub async fn zip_dir<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let mut file = fs::File::create(&output_path)
        .await
        .context("failed to create file")?;
    let mut writer = ZipFileWriter::with_tokio(&mut file);

    let mut entries = fs::read_dir(input_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        let mut buffer = vec![0; 1024 * 32];

        let mut file = fs::File::open(entry.path()).await?;

        let builder = ZipEntryBuilder::new(
            entry.file_name().to_string_lossy().to_string().into(),
            Compression::Stored,
        )
        .last_modification_date(build_zip_date_time(&file).await);

        let mut entry_writer = writer.write_entry_stream(builder).await?;

        loop {
            let size = file.read(&mut buffer).await?;

            if size == 0 {
                break;
            }

            entry_writer.write_all(&buffer[..size]).await?;
        }

        entry_writer
            .close()
            .await
            .context("failed to close entry")?;
    }

    writer.close().await.context("failed to close zip file")?;
    file.shutdown().await.context("failed to shutdown file")?;

    Ok(())
}

async fn build_zip_date_time(file: &File) -> ZipDateTime {
    let sys_time = file.metadata().await.unwrap().modified().unwrap();
    let duration = sys_time.duration_since(UNIX_EPOCH).unwrap();
    let secs = duration.as_secs();
    let nsecs = duration.subsec_nanos();

    let local = Local::now();
    let date = DateTime::from_timestamp(secs as i64, nsecs)
        .unwrap()
        .with_timezone(&local.timezone());

    ZipDateTimeBuilder::new()
        .year(date.year())
        .month(date.month())
        .day(date.day())
        .hour(date.hour())
        .minute(date.minute())
        .second(date.second())
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::LOGS_PATH;

    #[tokio::test]
    async fn test_zip_dir() {
        zip_dir(LOGS_PATH, "./logs.zip").await.unwrap();
    }
}
