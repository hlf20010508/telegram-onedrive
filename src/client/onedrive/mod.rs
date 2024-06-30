/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod session;

use grammers_client::types::Message;
use onedrive_api::{
    Auth, ClientCredential, DriveLocation, OneDrive as Client, Permission, Tenant, TokenResponse,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use session::OneDriveSession;

use super::utils::{socketio_client, socketio_disconnect};
use crate::auth_server::OD_CODE_EVENT;
use crate::env::{Env, OneDriveEnv};
use crate::error::{Error, Result};

pub struct OneDriveClient {
    client: RwLock<Client>,
    session: RwLock<OneDriveSession>,
    auth_provider: Auth,
    pub default_root_path: String,
    temp_root_path: RwLock<String>,
}

impl OneDriveClient {
    pub async fn new(
        Env {
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
        }: &Env,
    ) -> Self {
        let client = RwLock::new(Client::new("", DriveLocation::me()));
        let session = RwLock::new(OneDriveSession::default());
        let auth_provider = Auth::new(
            client_id,
            Permission::new_read().write(true).offline_access(true),
            format!("{}/auth", server_uri),
            Tenant::Common,
        );

        let onedrive_client = Self {
            client,
            session,
            auth_provider,
            default_root_path: root_path.to_string(),
            temp_root_path: RwLock::new("".to_string()),
        };

        let _ = onedrive_client
            .auto_login(session_path, client_secret)
            .await;

        onedrive_client
    }

    pub async fn login(
        &self,
        message: Arc<Message>,
        Env {
            onedrive:
                OneDriveEnv {
                    client_secret,
                    session_path,
                    root_path,
                    ..
                },
            port,
            use_reverse_proxy,
            ..
        }: &Env,
    ) -> Result<()> {
        if self.is_authorized().await || self.auto_login(session_path, client_secret).await.is_ok()
        {
            return Ok(());
        }

        let response = format!(
            "Here are the authorization url of OneDrive:\n\n{}",
            self.get_auth_url()
        );
        message
            .respond(response.as_str())
            .await
            .map_err(|e| Error::respond_error(e, response))?;

        let (socketio_client, mut rx) =
            socketio_client(OD_CODE_EVENT, port.to_owned(), use_reverse_proxy.to_owned()).await?;

        let code = rx
            .recv()
            .await
            .ok_or_else(|| Error::new("failed to receive onedrive code"))?;

        socketio_disconnect(socketio_client).await?;

        let response = "Code received, Authorizing...";
        message
            .respond(response)
            .await
            .map_err(|e| Error::respond_error(e, response))?;

        let TokenResponse {
            expires_in_secs,
            access_token,
            refresh_token,
            ..
        } = self
            .auth_provider
            .login_with_code(&code, &ClientCredential::Secret(client_secret.to_string()))
            .await
            .map_err(|e| {
                Error::context(
                    e,
                    "failed to get onedrive token response when login with code",
                )
            })?;

        *self.client.write().await = Client::new(&access_token, DriveLocation::me());

        let refresh_token = refresh_token.ok_or_else(|| {
            Error::new("failed to receive onedrive refresh token when login with code")
        })?;

        let session = OneDriveSession::new(
            &*self.client.read().await,
            expires_in_secs,
            &access_token,
            &refresh_token,
            session_path,
            root_path,
        )
        .await?;

        session.save().await?;

        session.set_current_user().await?;

        *self.session.write().await = session;

        Ok(())
    }

    async fn auto_login(&self, session_path: &str, client_secret: &str) -> Result<()> {
        let mut session = OneDriveSession::load(session_path).await?;

        let token_response = self
            .auth_provider
            .login_with_refresh_token(
                &session.refresh_token,
                &ClientCredential::Secret(client_secret.to_string()),
            )
            .await
            .map_err(|e| {
                Error::context(
                    e,
                    "failed to get refresh token response when login with refresh token",
                )
            })?;

        session.refresh_token = token_response.refresh_token.ok_or_else(|| {
            Error::new("failed to receive onedrive refresh token when login with refresh token")
        })?;
        session.access_token = token_response.access_token;
        session.set_expiration_timestamp(token_response.expires_in_secs);
        session.save().await?;

        *self.session.write().await = session;
        *self.client.write().await =
            Client::new(&self.session.read().await.access_token, DriveLocation::me());

        Ok(())
    }

    pub fn get_auth_url(&self) -> String {
        self.auth_provider.code_auth_url().to_string()
    }

    pub async fn is_authorized(&self) -> bool {
        self.client.read().await.get_drive().await.is_ok()
    }

    pub async fn get_root_path(&self, should_consume_temp: bool) -> Result<String> {
        let temp_root_path_read = self.temp_root_path.read().await;
        let temp_root_path_exists = self.does_temp_root_path_exist().await;

        let root_path = if should_consume_temp && temp_root_path_exists {
            let temp_root_path = temp_root_path_read.clone();
            self.clear_temp_root_path().await?;

            temp_root_path
        } else if !should_consume_temp && temp_root_path_exists {
            temp_root_path_read.clone()
        } else {
            self.session.read().await.root_path.clone()
        };

        Ok(root_path)
    }

    pub async fn does_temp_root_path_exist(&self) -> bool {
        !self.temp_root_path.read().await.is_empty()
    }

    pub async fn set_root_path(&self, path: &str) -> Result<()> {
        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path = path.to_string();
        session.save().await?;

        Ok(())
    }

    pub async fn reset_root_path(&self) -> Result<()> {
        self.clear_temp_root_path().await?;

        let mut session = self.session.write().await;
        session.root_path = self.default_root_path.clone();
        session.save().await?;

        Ok(())
    }

    pub async fn set_temp_root_path(&self, path: &str) -> Result<()> {
        *self.temp_root_path.write().await = path.to_string();

        Ok(())
    }

    pub async fn clear_temp_root_path(&self) -> Result<()> {
        self.set_temp_root_path("").await
    }
}