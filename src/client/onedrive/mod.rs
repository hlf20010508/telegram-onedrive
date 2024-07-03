/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod dir;
mod drive;
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
    ) -> Result<Self> {
        let client = RwLock::new(Client::new("", DriveLocation::me()));
        let session = RwLock::new(
            OneDriveSession::default()
                .set_connection(session_path)
                .await?,
        );
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

        Ok(onedrive_client)
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
        should_add: bool,
    ) -> Result<()> {
        if !should_add
            && (self.is_authorized().await
                || self.auto_login(session_path, client_secret).await.is_ok())
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

        let response = "Code received, authorizing...";
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

        let refresh_token = refresh_token.ok_or_else(|| {
            Error::new("failed to receive onedrive refresh token when login with code")
        })?;

        let client = Client::new(&access_token, DriveLocation::me());

        let session = OneDriveSession::new(
            &client,
            expires_in_secs,
            &access_token,
            &refresh_token,
            session_path,
            root_path,
        )
        .await?;

        session.save().await?;

        match self.get_current_username().await? {
            Some(username) => {
                if username == session.username {
                    self.session.write().await.overwrite(session).await?;
                    *self.client.write().await = client;
                }
            }
            None => {
                session.set_current_user().await?;
                self.session.write().await.overwrite(session).await?;
                *self.client.write().await = client;
            }
        }

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

        let access_token = token_response.access_token;
        *self.client.write().await = Client::new(&access_token, DriveLocation::me());

        session.refresh_token = token_response.refresh_token.ok_or_else(|| {
            Error::new("failed to receive onedrive refresh token when login with refresh token")
        })?;
        session.access_token = access_token;
        session.set_expiration_timestamp(token_response.expires_in_secs);
        session.save().await?;

        self.session.write().await.overwrite(session).await?;

        Ok(())
    }

    pub fn get_auth_url(&self) -> String {
        self.auth_provider.code_auth_url().to_string()
    }

    pub async fn is_authorized(&self) -> bool {
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
}
