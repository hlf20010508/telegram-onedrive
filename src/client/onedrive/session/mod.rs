/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;

use axum::http::header;
use onedrive_api::OneDrive;
use proc_macros::{add_context, add_trace};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ColumnTrait, ConnectionTrait, DatabaseConnection, EntityName, EntityTrait, ModelTrait,
    QueryFilter, QuerySelect, Schema, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use models::{current_user, session};

use crate::error::{Error, Result};
use crate::utils::get_current_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OneDriveSession {
    pub username: String,
    pub expiration_timestamp: i64,
    pub access_token: String,
    pub refresh_token: String,
    pub root_path: String,
    #[serde(skip)]
    connection: DatabaseConnection,
}

impl OneDriveSession {
    #[add_context]
    pub async fn new(
        client: &OneDrive,
        expires_in_secs: u64,
        access_token: &str,
        refresh_token: &str,
        session_path: &str,
        root_path: &str,
    ) -> Result<Self> {
        let username = Self::get_username(client).await?;
        let expiration_timestamp = Self::get_expiration_timestamp(expires_in_secs);
        let connection = Self::connect_db(session_path).await?;

        Ok(Self {
            username,
            expiration_timestamp,
            access_token: access_token.to_string(),
            refresh_token: refresh_token.to_string(),
            root_path: root_path.to_string(),
            connection,
        })
    }

    #[add_trace]
    fn get_expiration_timestamp(expires_in_secs: u64) -> i64 {
        let expiration_timestamp = get_current_timestamp() + expires_in_secs as i64;

        tracing::debug!(
            "onedrive session expiration_timestamp: {}",
            expiration_timestamp
        );

        expiration_timestamp
    }

    #[add_trace]
    pub fn set_expiration_timestamp(&mut self, expires_in_secs: u64) {
        self.expiration_timestamp = Self::get_expiration_timestamp(expires_in_secs);

        tracing::debug!(
            "set onedrive session expiration_timestamp to {}",
            self.expiration_timestamp
        );
    }

    #[add_context]
    #[add_trace]
    async fn get_username(client: &OneDrive) -> Result<String> {
        let http_client = client.client();

        let url = "https://graph.microsoft.com/v1.0/me/";

        let response = http_client
            .get(url)
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", client.access_token()),
            )
            .send()
            .await
            .map_err(|e| Error::new("failed to send request for user profile").raw(e))?;

        let content = response
            .text()
            .await
            .map_err(|e| Error::new("failed to get response text for user profile").raw(e))?;

        let user_profile = serde_json::from_str::<Value>(&content)
            .map_err(|e| Error::new("failed to deserialize user profile into Value").raw(e))?;

        let username = user_profile
            .get("userPrincipalName")
            .ok_or_else(|| Error::new("field userPrincipalName not found in user profile"))?
            .as_str()
            .ok_or_else(|| Error::new("userPrincipalName value is not a string"))?
            .to_string();

        tracing::debug!("got onedrive username: {}", username);

        Ok(username)
    }

    #[add_context]
    #[add_trace]
    async fn connect_db(path: &str) -> Result<DatabaseConnection> {
        let connection = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path))
            .await
            .map_err(|e| Error::new("failed to connect to onedrive session").raw(e))?;

        Self::create_table_if_not_exists(&connection, session::Entity).await?;
        Self::create_table_if_not_exists(&connection, current_user::Entity).await?;

        Ok(connection)
    }

    #[add_context]
    #[add_trace]
    pub async fn set_connection(mut self, session_path: &str) -> Result<Self> {
        self.connection = Self::connect_db(session_path).await?;

        Ok(self)
    }

    #[add_trace]
    async fn is_table_exists<E>(connection: &DatabaseConnection) -> bool
    where
        E: EntityTrait,
    {
        let result = E::find().all(connection).await;

        result.is_ok()
    }

    #[add_context]
    #[add_trace]
    async fn create_table_if_not_exists<E>(connection: &DatabaseConnection, entity: E) -> Result<()>
    where
        E: EntityTrait + EntityName,
    {
        if Self::is_table_exists::<E>(connection).await {
            tracing::debug!(
                "onedrive session database table {} already exists",
                entity.table_name()
            );
        } else {
            tracing::debug!(
                "onedrive session database table {} not exists, create it",
                entity.table_name()
            );

            let backend = connection.get_database_backend();

            let table_create_statement = Schema::new(backend).create_table_from_entity(entity);

            connection
                .execute(backend.build(&table_create_statement))
                .await
                .map_err(|e| {
                    Error::new(format!("failed to create table {}", entity.table_name())).raw(e)
                })?;
        }

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn load(path: &str) -> Result<Self> {
        tracing::debug!("load onedrive session");

        let connection = Self::connect_db(path).await?;

        let mut session = Self::from(Self::get_current_session(&connection).await?);

        session.connection = connection;

        Ok(session)
    }

    #[add_context]
    #[add_trace]
    pub async fn save(&self) -> Result<()> {
        tracing::debug!("save onedrive session");

        if self.user_exists().await? {
            tracing::debug!("onedrive session user already exists, update it");

            self.update().await?;
        } else {
            tracing::debug!("onedrive session user not exists, insert it");

            let insert_item = session::ActiveModel {
                username: Set(self.username.to_string()),
                expiration_timestamp: Set(self.expiration_timestamp),
                access_token: Set(self.access_token.to_string()),
                refresh_token: Set(self.refresh_token.to_string()),
                root_path: Set(self.root_path.to_string()),
            };

            session::Entity::insert(insert_item)
                .exec(&self.connection)
                .await
                .map_err(|e| Error::new("failed to insert onedrive session").raw(e))?;
        }

        Ok(())
    }

    #[add_trace]
    pub fn overwrite(
        &mut self,
        Self {
            username,
            expiration_timestamp,
            access_token,
            refresh_token,
            root_path,
            ..
        }: Self,
    ) {
        self.username = username;
        self.expiration_timestamp = expiration_timestamp;
        self.access_token = access_token;
        self.refresh_token = refresh_token;
        self.root_path = root_path;
    }

    #[add_context]
    #[add_trace]
    async fn user_exists(&self) -> Result<bool> {
        let exists = session::Entity::find()
            .filter(session::Column::Username.eq(&self.username))
            .one(&self.connection)
            .await
            .map_err(|e| Error::new("failed to query onedrive session").raw(e))?
            .is_some();

        tracing::debug!("user {} exists: {}", self.username, exists);

        Ok(exists)
    }

    #[add_context]
    #[add_trace]
    async fn update(&self) -> Result<()> {
        session::Entity::update_many()
            .filter(session::Column::Username.eq(&self.username))
            .col_expr(
                session::Column::ExpirationTimestamp,
                Expr::value(self.expiration_timestamp),
            )
            .col_expr(
                session::Column::AccessToken,
                Expr::value(&self.access_token),
            )
            .col_expr(
                session::Column::RefreshToken,
                Expr::value(&self.refresh_token),
            )
            .col_expr(session::Column::RootPath, Expr::value(&self.root_path))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to update onedrive session").raw(e))?;

        tracing::debug!("updated onedrive session for user {}", self.username);

        Ok(())
    }

    #[add_context]
    #[add_trace]
    async fn get_current_session(connection: &DatabaseConnection) -> Result<session::Model> {
        let current_user = current_user::Entity::find()
            .one(connection)
            .await
            .map_err(|e| Error::new("faield to query onedrive current user").raw(e))?
            .ok_or_else(|| Error::new("onedrive current user not found"))?;

        let session = current_user
            .find_related(session::Entity)
            .one(connection)
            .await
            .map_err(|e| Error::new("failed to query related onedrive session").raw(e))?
            .ok_or_else(|| Error::new("related onedrive session not found"))?;

        tracing::debug!(
            "got onedrive current session for user {}",
            current_user.username
        );

        Ok(session)
    }

    #[add_context]
    #[add_trace]
    pub async fn set_current_user(&self) -> Result<()> {
        tracing::debug!("onedrive session user to be set: {}", self.username);

        let current_user_col = current_user::Entity::find()
            .one(&self.connection)
            .await
            .map_err(|e| Error::new("failed to query onedrive current user").raw(e))?;

        if let Some(current_user_col) = current_user_col {
            tracing::debug!(
                "onedrive session current user: {}",
                current_user_col.username
            );

            if current_user_col.username == self.username {
                tracing::debug!(
                    "onedrive session user to be set is the same as current user, skip"
                );
                return Ok(());
            }

            tracing::debug!("onedrive session user to be set is different from current user");

            current_user::Entity::delete_many()
                .exec(&self.connection)
                .await
                .map_err(|e| Error::new("failed to delete onedrive current user").raw(e))?;
        }

        let insert_item = current_user::ActiveModel {
            username: Set(self.username.clone()),
        };

        current_user::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to insert onedrive current user").raw(e))?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn get_usernames(&self) -> Result<Vec<String>> {
        let result = session::Entity::find()
            .column(session::Column::Username)
            .all(&self.connection)
            .await
            .map_err(|e| Error::new("failed to query onedrive usernames").raw(e))?;

        let usernames = result
            .into_iter()
            .map(|row| row.username)
            .collect::<Vec<String>>();

        tracing::debug!("got onedrive usernames: {:#?}", usernames);
        Ok(usernames)
    }

    #[add_context]
    #[add_trace]
    pub async fn get_current_username(&self) -> Result<Option<String>> {
        match current_user::Entity::find()
            .one(&self.connection)
            .await
            .map_err(|e| Error::new("failed to query onedrive current username").raw(e))?
        {
            Some(model) => {
                tracing::debug!("got onedrive current username: {}", model.username);

                Ok(Some(model.username))
            }
            None => {
                tracing::debug!("no onedrive current username found");

                Ok(None)
            }
        }
    }

    #[add_context]
    #[add_trace]
    pub async fn remove_user(&mut self, username: Option<String>) -> Result<()> {
        tracing::debug!("onedrive user to be removed: {:?}", username);
        tracing::debug!("onedrive current user: {}", self.username);

        let username = match username {
            Some(username) => username,
            None => self.username.clone(),
        };

        if username == self.username {
            tracing::debug!(
                "onedrive user to be removed is the current user, remove it in table current_user"
            );

            current_user::Entity::delete_many()
                .exec(&self.connection)
                .await
                .map_err(|e| Error::new("failed to delete onedrive current user").raw(e))?;
        }

        tracing::debug!("remove onedrive user in table session");

        session::Entity::delete_many()
            .filter(session::Column::Username.eq(&username))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to delete onedrive session").raw(e))?;

        if username != self.username {
            return Ok(());
        }

        tracing::debug!("onedrive user removed is the current user, set a new one");

        match session::Entity::find().one(&self.connection).await {
            Ok(Some(session)) => {
                tracing::debug!("new onedrive user: {}", session.username);

                self.overwrite(Self::from(session));

                self.set_current_user().await?;
            }
            Ok(None) => {
                tracing::debug!("no new onedrive user found, set session to empty");

                let session = Self::default();

                self.overwrite(session);
            }
            Err(e) => return Err(Error::new("failed to query onedrive session").raw(e)),
        }

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn change_session(&mut self, username: &str) -> Result<()> {
        tracing::debug!("onedrive session to change is {}", username);

        if username == self.username {
            tracing::debug!("onedrive user to change is the current user, skip");

            return Ok(());
        }

        let session = session::Entity::find()
            .filter(session::Column::Username.eq(username))
            .one(&self.connection)
            .await
            .map_err(|e| Error::new("failed to query onedrive session").raw(e))?
            .ok_or_else(|| Error::new("onedrive session not found"))?;

        tracing::debug!(
            "change onedrive session user from {} to {}",
            self.username,
            session.username
        );

        self.overwrite(Self::from(session));

        self.set_current_user().await?;

        Ok(())
    }

    #[add_trace]
    pub fn is_expired(&self) -> bool {
        let is_expired = self.expiration_timestamp < get_current_timestamp() + 60;

        tracing::debug!("onedrive session is expired: {}", is_expired);

        is_expired
    }
}

impl From<session::Model> for OneDriveSession {
    fn from(model: session::Model) -> Self {
        Self {
            username: model.username,
            expiration_timestamp: model.expiration_timestamp,
            access_token: model.access_token,
            refresh_token: model.refresh_token,
            root_path: model.root_path,
            connection: DatabaseConnection::default(),
        }
    }
}
