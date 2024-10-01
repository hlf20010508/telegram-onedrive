/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use crate::error::{Error, Result};
use path_slash::PathExt;
use proc_macros::{add_context, add_trace};
use std::{
    io::{Read, Write},
    path::Path,
};
use zip::write::{FileOptions, SimpleFileOptions};

#[add_context]
#[add_trace]
pub fn zip_dir<P: AsRef<Path>>(input_path: P, output_path: P) -> Result<()> {
    let zip_file = std::fs::File::create(&output_path)
        .map_err(|e| Error::new("failed to create file").raw(e))?;
    let mut zip = zip::ZipWriter::new(zip_file);

    let options = SimpleFileOptions::default();

    add_entry(input_path.as_ref(), input_path.as_ref(), &mut zip, options)?;

    zip.finish()
        .map_err(|e| Error::new("failed to finish zip").raw(e))?;

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
            .map_err(|e| Error::new("failed to start zip file").raw(e))?;

        let mut file =
            std::fs::File::open(path).map_err(|e| Error::new("failed to open file").raw(e))?;

        loop {
            let size = file
                .read(&mut buffer)
                .map_err(|e| Error::new("failed to read file").raw(e))?;

            if size == 0 {
                break;
            }

            zip.write_all(&buffer[..size])
                .map_err(|e| Error::new("failed to write file").raw(e))?;
        }
    } else if path.is_dir() {
        zip.add_directory(name.to_slash_lossy(), options)
            .map_err(|e| Error::new("failed to add directory to zip").raw(e))?;

        for entry in std::fs::read_dir(path)
            .map_err(|e| Error::new("failed to read dir").raw(e).context("sub dir"))?
        {
            let entry = entry.map_err(|e| {
                Error::new("failed to read dir entry")
                    .raw(e)
                    .context("sub dir entry")
            })?;

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