/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

mod models;

use axum::http::header;
use onedrive_api::OneDrive;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    fn get_expiration_timestamp(expires_in_secs: u64) -> i64 {
        get_current_timestamp() + expires_in_secs as i64
    }

    pub fn set_expiration_timestamp(&mut self, expires_in_secs: u64) {
        self.expiration_timestamp = Self::get_expiration_timestamp(expires_in_secs);
    }

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
            .map_err(|e| Error::context(e, "failed to send request for user profile"))?;

        let content = response
            .text()
            .await
            .map_err(|e| Error::context(e, "failed to get response text for user profile"))?;

        let user_profile = serde_json::from_str::<Value>(&content)
            .map_err(|e| Error::context(e, "failed to deserialize user profile into Value"))?;

        let username = user_profile
            .get("userPrincipalName")
            .ok_or_else(|| Error::new("field userPrincipalName not found in user profile"))?
            .as_str()
            .ok_or_else(|| Error::new("userPrincipalName value is not a string"))?
            .to_string();

        Ok(username)
    }

    async fn connect_db(path: &str) -> Result<DatabaseConnection> {
        let connection = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path))
            .await
            .map_err(|e| Error::context(e, "failed to connect to onedrive session"))?;

        Self::create_table_if_not_exists(&connection, session::Entity).await?;
        Self::create_table_if_not_exists(&connection, current_user::Entity).await?;

        Ok(connection)
    }

    pub async fn set_connection(mut self, session_path: &str) -> Result<Self> {
        self.connection = Self::connect_db(session_path).await?;

        Ok(self)
    }

    async fn is_table_exists<E>(connection: &DatabaseConnection) -> bool
    where
        E: EntityTrait,
    {
        let result = E::find().all(connection).await;

        if result.is_ok() {
            true
        } else {
            false
        }
    }

    async fn create_table_if_not_exists<E>(connection: &DatabaseConnection, entity: E) -> Result<()>
    where
        E: EntityTrait + EntityName,
    {
        if !Self::is_table_exists::<E>(connection).await {
            let backend = connection.get_database_backend();

            let table_create_statement = Schema::new(backend).create_table_from_entity(entity);

            connection
                .execute(backend.build(&table_create_statement))
                .await
                .map_err(|e| {
                    Error::context(e, format!("failed to create table {}", entity.table_name()))
                })?;
        }

        Ok(())
    }

    pub async fn load(path: &str) -> Result<Self> {
        let connection = Self::connect_db(path).await?;

        let mut session = Self::from(Self::get_current_session(&connection).await?);

        session.connection = connection;

        Ok(session)
    }

    pub async fn save(&self) -> Result<()> {
        if self.user_exists().await? {
            self.update().await?;
        } else {
            let insert_item = session::ActiveModel {
                username: Set(self.username.to_string()),
                expiration_timestamp: Set(self.expiration_timestamp),
                access_token: Set(self.access_token.to_string()),
                refresh_token: Set(self.refresh_token.to_string()),
                root_path: Set(self.root_path.to_string()),
                ..Default::default()
            };

            session::Entity::insert(insert_item)
                .exec(&self.connection)
                .await
                .map_err(|e| Error::context(e, "failed to insert onedrive session"))?;
        }

        Ok(())
    }

    pub async fn overwrite(
        &mut self,
        Self {
            username,
            expiration_timestamp,
            access_token,
            refresh_token,
            root_path,
            ..
        }: Self,
    ) -> Result<()> {
        self.username = username;
        self.expiration_timestamp = expiration_timestamp;
        self.access_token = access_token;
        self.refresh_token = refresh_token;
        self.root_path = root_path;

        Ok(())
    }

    async fn user_exists(&self) -> Result<bool> {
        let exists = session::Entity::find()
            .filter(session::Column::Username.eq(&self.username))
            .one(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to query onedrive session"))?
            .is_some();

        Ok(exists)
    }

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
            .map_err(|e| Error::context(e, "failed to update onedrive session"))?;

        Ok(())
    }

    async fn get_current_session(connection: &DatabaseConnection) -> Result<session::Model> {
        let current_user = current_user::Entity::find()
            .one(connection)
            .await
            .map_err(|e| Error::context(e, "faield to query onedrive current user"))?
            .ok_or_else(|| Error::new("onedrive current user not found"))?;

        let session = current_user
            .find_related(session::Entity)
            .one(connection)
            .await
            .map_err(|e| Error::context(e, "failed to query related onedrive session"))?
            .ok_or_else(|| Error::new("related onedrive session not found"))?;

        Ok(session)
    }

    pub async fn set_current_user(&self) -> Result<()> {
        let current_user_col = current_user::Entity::find()
            .one(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to query onedrive current user"))?;

        if let Some(current_user_col) = current_user_col {
            if current_user_col.username != self.username {
                current_user::Entity::delete_many()
                    .exec(&self.connection)
                    .await
                    .map_err(|e| Error::context(e, "failed to delete onedrive current user"))?;
            } else {
                return Ok(());
            }
        }

        let insert_item = current_user::ActiveModel {
            username: Set(self.username.clone()),
            ..Default::default()
        };

        current_user::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to insert onedrive current user"))?;

        Ok(())
    }

    pub async fn get_usernames(&self) -> Result<Vec<String>> {
        let result = session::Entity::find()
            .column(session::Column::Username)
            .all(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to query onedrive usernames"))?;

        let usernames = result
            .into_iter()
            .map(|row| row.username)
            .collect::<Vec<String>>();

        Ok(usernames)
    }

    pub async fn get_current_username(&self) -> Result<Option<String>> {
        match current_user::Entity::find()
            .one(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to query onedrive current username"))?
        {
            Some(model) => Ok(Some(model.username)),
            None => Ok(None),
        }
    }

    pub async fn remove_user(&mut self, username: Option<String>) -> Result<()> {
        let username = match username {
            Some(username) => username,
            None => self.username.clone(),
        };

        if username == self.username {
            current_user::Entity::delete_many()
                .exec(&self.connection)
                .await
                .map_err(|e| Error::context(e, "failed to delete onedrive current user"))?;
        }

        session::Entity::delete_many()
            .filter(session::Column::Username.eq(&username))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to delete onedrive session"))?;

        if username != self.username {
            return Ok(());
        }

        match session::Entity::find().one(&self.connection).await {
            Ok(Some(session)) => {
                self.overwrite(Self::from(session)).await?;

                self.set_current_user().await?;
            }
            Ok(None) => {
                let session = Self::default();

                self.overwrite(session).await?;
            }
            Err(e) => return Err(Error::context(e, "failed to query onedrive session")),
        }

        Ok(())
    }

    pub async fn change_session(&mut self, username: &str) -> Result<()> {
        if username == self.username {
            return Ok(());
        }

        let session = session::Entity::find()
            .filter(session::Column::Username.eq(username))
            .one(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to query onedrive session"))?
            .ok_or_else(|| Error::new("onedrive session not found"))?;

        self.overwrite(Self::from(session)).await?;

        self.set_current_user().await?;

        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        self.expiration_timestamp < get_current_timestamp() + 60
    }
}

impl Default for OneDriveSession {
    fn default() -> Self {
        Self {
            username: Default::default(),
            expiration_timestamp: Default::default(),
            access_token: Default::default(),
            refresh_token: Default::default(),
            root_path: Default::default(),
            connection: Default::default(),
        }
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
            connection: Default::default(),
        }
    }
}
