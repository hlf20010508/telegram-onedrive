/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{Context, Result};
use path_slash::PathExt;
use std::{
    io::{Read, Write},
    path::Path,
};
use zip::write::{FileOptions, SimpleFileOptions};

pub fn zip_dir<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let zip_file = std::fs::File::create(&output_path).context("failed to create file")?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default();

    add_entry(input_path.as_ref(), input_path.as_ref(), &mut zip, options)?;

    zip.finish().context("failed to finish zip")?;

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
        .context("failed to strip prefix")?;

    // path seperator in zip must be slash /, not backslash \

    if path.is_file() {
        zip.start_file(name.to_slash_lossy(), options)
            .context("failed to start zip file")?;

        let mut file = std::fs::File::open(path).context("failed to open file")?;

        loop {
            let size = file.read(&mut buffer).context("failed to read file")?;

            if size == 0 {
                break;
            }

            zip.write_all(&buffer[..size])
                .context("failed to write file")?;
        }
    } else if path.is_dir() {
        zip.add_directory(name.to_slash_lossy(), options)
            .context("failed to add directory to zip")?;

        for entry in std::fs::read_dir(path)
            .context("failed to read dir")
            .context("sub dir")?
        {
            let entry = entry
                .context("failed to read dir entry")
                .context("sub dir entry")?;

            add_entry(base_path, &entry.path(), zip, options)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::LOGS_PATH;

    #[test]
    fn test_zip_dir() {
        zip_dir(LOGS_PATH, "./logs.zip").unwrap();
    }
}
