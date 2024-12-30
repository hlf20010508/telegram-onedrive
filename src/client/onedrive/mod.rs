/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod dir;
mod drive;
pub mod invalid_name;
mod session;
mod upload;
mod utils;

use crate::{
    env::{Env, OneDriveEnv, ENV},
    message::TelegramMessage,
};
use anyhow::{anyhow, Context, Result};
use onedrive_api::{
    Auth, ClientCredential, DriveLocation, OneDrive as Client, Permission, Tenant, TokenResponse,
};
use path_slash::PathBufExt;
use session::OneDriveSession;
use std::path::Path;
use tokio::sync::{mpsc::Receiver, RwLock};

pub struct OneDriveClient {
    client: RwLock<Client>,
    session: RwLock<OneDriveSession>,
    auth_provider: Auth,
    client_secret: String,
    session_path: String,
    pub default_root_path: String,
    temp_root_path: RwLock<String>,
}

impl OneDriveClient {
    pub async fn new() -> Result<Self> {
        let Env {
            onedrive:
                OneDriveEnv {
                    client_id,
                    client_secret,
                    session_path,
                    root_path,
                    ..
                },
            server_uri,
            ..
        } = ENV.get().unwrap();

        let client = RwLock::new(Client::new("", DriveLocation::me()));
        let session = RwLock::new(
            OneDriveSession::default()
                .set_connection(session_path)
                .await?,
        );
        let auth_provider = Auth::new(
            client_id,
            Permission::new_read()
                .write(true)
                .access_shared(true)
                .offline_access(true),
            Path::new(server_uri).join("auth").to_slash_lossy(),
            Tenant::Common,
        );

        let onedrive_client = Self {
            client,
            session,
            auth_provider,
            client_secret: client_secret.clone(),
            session_path: session_path.clone(),
            default_root_path: root_path.to_string(),
            temp_root_path: RwLock::new(String::new()),
        };

        let _ = onedrive_client.auto_login().await;

        Ok(onedrive_client)
    }

    pub async fn login(
        &self,
        message: TelegramMessage,
        should_add: bool,
        mut rx: Receiver<String>,
    ) -> Result<()> {
        tracing::info!("logging in to onedrive");

        if !should_add {
            tracing::debug!("onedrive account should not be added");

            if self.is_authorized().await {
                tracing::debug!("onedrive account has been authorized");

                return Ok(());
            }

            tracing::info!("onedrive account is not authorized, auto login");

            if self.auto_login().await.is_ok() {
                tracing::info!("onedrive auto login successful");

                return Ok(());
            }

            tracing::info!("onedrive auto login failed, login manually");
        }

        let response = format!(
            "Here are the authorization url of OneDrive:\n\n{}",
            self.get_auth_url()
        );
        message.respond(response.as_str()).await.context(response)?;

        tracing::info!("onedrive authorization url sent");

        let code = rx
            .recv()
            .await
            .ok_or_else(|| anyhow!("failed to receive onedrive code"))?;

        tracing::info!("onedrive code received");
        tracing::debug!("onedrive code: {}", code);

        let response = "Code received, authorizing...";
        message.respond(response).await.context(response)?;

        tracing::info!("onedrive authorizing");

        let TokenResponse {
            expires_in_secs,
            access_token,
            refresh_token,
            ..
        } = self
            .auth_provider
            .login_with_code(&code, &ClientCredential::Secret(self.client_secret.clone()))
            .await
            .context("failed to get onedrive token response when login with code")?;

        let refresh_token = refresh_token.ok_or_else(|| {
            anyhow!("failed to receive onedrive refresh token when login with code")
        })?;

        let client = Client::new(&access_token, DriveLocation::me());

        tracing::info!("onedrive authorized");

        let session = OneDriveSession::new(
            expires_in_secs,
            &access_token,
            &refresh_token,
            &self.session_path,
            &self.default_root_path,
        )
        .await?;

        session.save().await?;

        if let Some(username) = self.get_current_username().await? {
            if username == session.username {
                self.session.write().await.overwrite(session);
                *self.client.write().await = client;
            }
        } else {
            session.set_current_user().await?;
            self.session.write().await.overwrite(session);
            *self.client.write().await = client;
        }

        Ok(())
    }

    async fn auto_login(&self) -> Result<()> {
        let mut session = OneDriveSession::load(&self.session_path).await?;

        let token_response = self
            .get_token_using_refresh_token(&session.refresh_token)
            .await?;

        let access_token = token_response.access_token;
        *self.client.write().await = Client::new(&access_token, DriveLocation::me());

        session.refresh_token = token_response.refresh_token.ok_or_else(|| {
            anyhow!("failed to receive onedrive refresh token when login with refresh token")
        })?;
        session.access_token = access_token;
        session.set_expiration_timestamp(token_response.expires_in_secs);
        session.save().await?;

        self.session.write().await.overwrite(session);

        Ok(())
    }

    pub async fn get_token_using_refresh_token(
        &self,
        refresh_token: &str,
    ) -> Result<TokenResponse> {
        self.auth_provider
            .login_with_refresh_token(
                refresh_token,
                &ClientCredential::Secret(self.client_secret.clone()),
            )
            .await
            .context("failed to get refresh token response when login with refresh token")
    }

    pub fn get_auth_url(&self) -> String {
        let auth_url = self.auth_provider.code_auth_url().to_string();

        tracing::info!("onedrive auth url: {}", auth_url);

        auth_url
    }

    pub async fn is_authorized(&self) -> bool {
        let is_expired = { self.session.read().await.is_expired() };

        if is_expired {
            self.refresh_access_token().await.ok();
        }

        self.client.read().await.get_drive().await.is_ok()
    }

    pub async fn set_current_user(&self) -> Result<()> {
        self.session.write().await.set_current_user().await
    }

    pub async fn logout(&self, username: Option<String>) -> Result<()> {
        let mut session = self.session.write().await;
        session.remove_user(username).await?;

        *self.client.write().await = Client::new(&session.access_token, DriveLocation::me());

        Ok(())
    }

    pub async fn refresh_access_token(&self) -> Result<()> {
        let is_expired = { self.session.read().await.is_expired() };

        if is_expired {
            let mut session = self.session.write().await;

            let token_response = self
                .get_token_using_refresh_token(&session.refresh_token)
                .await?;

            session.access_token = token_response.access_token;
            session.refresh_token = token_response.refresh_token.ok_or_else(|| {
                anyhow!("failed to receive onedrive refresh token when login with refresh token")
            })?;
            session.set_expiration_timestamp(token_response.expires_in_secs);

            session.save().await?;

            *self.client.write().await =
                Client::new(session.access_token.clone(), DriveLocation::me());
        }

        Ok(())
    }
}
