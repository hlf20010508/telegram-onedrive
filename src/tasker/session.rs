/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use std::collections::HashMap;

use sea_orm::{
    sea_query::Expr, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection, EntityName,
    EntityTrait, PaginatorTrait, QueryFilter, Schema, Set,
};

use super::tasks::{self, CmdType, TaskStatus};
use crate::error::{Error, Result};

pub struct TaskSession {
    connection: DatabaseConnection,
}

impl TaskSession {
    pub async fn new(session_path: &str) -> Result<Self> {
        let connection = Self::connect_db(session_path).await?;

        Ok(Self { connection })
    }

    async fn connect_db(path: &str) -> Result<DatabaseConnection> {
        let connection = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path))
            .await
            .map_err(|e| Error::context(e, "failed to connect to task session"))?;

        Self::create_table_if_not_exists(&connection).await?;

        Ok(connection)
    }

    async fn create_table_if_not_exists(connection: &DatabaseConnection) -> Result<()> {
        if !Self::is_table_exists(connection).await {
            let backend = connection.get_database_backend();

            let table_create_statement =
                Schema::new(backend).create_table_from_entity(tasks::Entity);

            connection
                .execute(backend.build(&table_create_statement))
                .await
                .map_err(|e| {
                    Error::context(
                        e,
                        format!("failed to create table {}", tasks::Entity.table_name()),
                    )
                })?;
        }

        Ok(())
    }

    async fn is_table_exists(connection: &DatabaseConnection) -> bool {
        let result = tasks::Entity::find().all(connection).await;

        if result.is_ok() {
            true
        } else {
            false
        }
    }

    pub async fn fetch_task(&self) -> Result<Option<tasks::Model>> {
        tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Waiting))
            .one(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to get a task"))
    }

    pub async fn insert_task(
        &self,
        cmd_type: CmdType,
        filename: &str,
        root_path: &str,
        url: &str,
        upload_url: &str,
        current_length: u64,
        total_length: u64,
        chat_bot_hex: &str,
        chat_user_hex: &str,
        message_id: i32,
    ) -> Result<()> {
        let insert_item = tasks::ActiveModel {
            cmd_type: Set(cmd_type),
            filename: Set(filename.to_string()),
            root_path: Set(root_path.to_string()),
            url: Set(url.to_string()),
            upload_url: Set(upload_url.to_string()),
            current_length: Set(current_length as i64),
            total_length: Set(total_length as i64),
            chat_bot_hex: Set(chat_bot_hex.to_string()),
            chat_user_hex: Set(chat_user_hex.to_string()),
            message_id: Set(message_id),
            status: Set(TaskStatus::Waiting),
            ..Default::default()
        };

        tasks::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to insert url task"))?;

        Ok(())
    }

    pub async fn set_task_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Status, Expr::value(status))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to update task status"))?;

        Ok(())
    }

    pub async fn set_current_length(&self, id: i64, current_length: u64) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(
                tasks::Column::CurrentLength,
                Expr::value(current_length as i64),
            )
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to update current length"))?;

        Ok(())
    }

    pub async fn get_chats_tasks(&self) -> Result<HashMap<ChatHex, ChatTasks>> {
        let mut chats = HashMap::new();

        macro_rules! insert_chat_tasks {
            ($status: ident, $task_type: ident) => {
                let tasks = tasks::Entity::find()
                    .filter(tasks::Column::Status.eq(TaskStatus::$status))
                    .all(&self.connection)
                    .await
                    .map_err(|e| Error::context(e, "failed to get chat current tasks"))?;

                for task in tasks {
                    chats
                        .entry(ChatHex {
                            chat_bot_hex: task.chat_bot_hex.clone(),
                            chat_user_hex: task.chat_user_hex.clone(),
                        })
                        .or_insert(ChatTasks {
                            current_tasks: Vec::new(),
                            completed_tasks: Vec::new(),
                            failed_tasks: Vec::new(),
                        })
                        .$task_type
                        .push(task);
                }
            };
        }

        insert_chat_tasks!(Started, current_tasks);
        insert_chat_tasks!(Completed, completed_tasks);
        insert_chat_tasks!(Failed, failed_tasks);

        Ok(chats)
    }

    pub async fn get_chat_pending_tasks_number(&self, chat_bot_hex: &str) -> Result<u64> {
        tasks::Entity::find()
            .filter(
                Condition::all()
                    .add(tasks::Column::ChatBotHex.eq(chat_bot_hex))
                    .add(tasks::Column::Status.eq(TaskStatus::Waiting)),
            )
            .count(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to get pending tasks number"))
    }

    pub async fn does_chat_has_started_tasks(&self, chat_bot_hex: &str) -> Result<bool> {
        let has_started_tasks = tasks::Entity::find()
            .filter(
                Condition::all()
                    .add(tasks::Column::ChatBotHex.eq(chat_bot_hex))
                    .add(tasks::Column::Status.eq(TaskStatus::Started)),
            )
            .count(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to get chat started tasks number"))?
            > 0;

        Ok(has_started_tasks)
    }

    pub async fn update_filename(&self, id: i64, filename: &str) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Filename, Expr::value(filename))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to update filename"))?;

        Ok(())
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        tasks::Entity::delete_by_id(id)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to delete task"))?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        tasks::Entity::delete_many()
            .exec(&self.connection)
            .await
            .map_err(|e| Error::context(e, "failed to clear tasks"))?;

        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct ChatHex {
    pub chat_bot_hex: String,
    pub chat_user_hex: String,
}

pub struct ChatTasks {
    pub current_tasks: Vec<tasks::Model>,
    pub completed_tasks: Vec<tasks::Model>,
    pub failed_tasks: Vec<tasks::Model>,
}
