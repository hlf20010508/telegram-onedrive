/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use std::path::Path;

pub async fn get_rustls_config() -> Result<RustlsConfig> {
    let cert_path = {
        let path = Path::new("ssl/server.crt");

        if path.exists() {
            path
        } else {
            Path::new("/telegram-onedrive/server/ssl/server.crt")
        }
    };
    let key_path = {
        let path = Path::new("ssl/server.key");

        if path.exists() {
            path
        } else {
            Path::new("/telegram-onedrive/server/ssl/server.key")
        }
    };

    let config = if cert_path.exists() && key_path.exists() {
        tracing::debug!("auth server uses cert from file");

        RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .context("failed to create rustls config from pem file")?
    } else {
        tracing::debug!("auth server uses self signed cert");

        let (cert, key) = gen_self_signed_cert()?;

        RustlsConfig::from_pem(cert, key)
            .await
            .context("failed to create self signed rustls config")?
    };

    Ok(config)
}

fn gen_self_signed_cert() -> Result<(Vec<u8>, Vec<u8>)> {
    let subject_alt_names = vec!["127.0.0.1".to_string(), "localhost".to_string()];

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names)
        .context("failed to generate self signed cert")?;

    Ok((
        cert.pem().into_bytes(),
        key_pair.serialize_pem().into_bytes(),
    ))
}
