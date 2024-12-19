/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::tasks::{self, InsertTask, TaskStatus};
use anyhow::{Context, Ok, Result};
use sea_orm::{
    sea_query::Expr, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    EntityName, EntityTrait, PaginatorTrait, QueryFilter, Schema, Set,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{fs, sync::Mutex};
use tokio_util::sync::CancellationToken;

// (chat id, message indicator id) -> aborter
pub type TaskAborters = Arc<Mutex<HashMap<(i64, i32), TaskAborter>>>;
pub type BatchAborters = Arc<Mutex<HashMap<(i64, i32), BatchAborter>>>;

pub struct TaskSession {
    connection: DatabaseConnection,
    pub task_aborters: TaskAborters,
    pub batch_aborters: BatchAborters,
}

impl TaskSession {
    pub async fn new(session_path: &str) -> Result<Self> {
        if Path::new(session_path).exists() {
            fs::remove_file(session_path)
                .await
                .context("failed to remove old task session")?;
        }

        let connection = Self::connect_db(session_path).await?;
        let task_aborters = Arc::new(Mutex::new(HashMap::new()));
        let batch_aborters = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            connection,
            task_aborters,
            batch_aborters,
        })
    }

    async fn connect_db(path: &str) -> Result<DatabaseConnection> {
        let connection = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path))
            .await
            .context("failed to connect to task session")?;

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
                .context(format!(
                    "failed to create table {}",
                    tasks::Entity.table_name()
                ))?;
        }

        Ok(())
    }

    async fn is_table_exists(connection: &DatabaseConnection) -> bool {
        let result = tasks::Entity::find().all(connection).await;

        result.is_ok()
    }

    pub async fn fetch_task(&self) -> Result<Option<tasks::Model>> {
        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Waiting))
            .one(&self.connection)
            .await
            .context("failed to get a task")?;

        if let Some(task) = &task {
            self.set_task_status(task.id, tasks::TaskStatus::Fetched)
                .await?;
        }

        Ok(task)
    }

    pub async fn insert_task(
        &self,
        InsertTask {
            cmd_type,
            filename,
            root_path,
            url,
            upload_url,
            current_length,
            total_length,
            chat_id,
            chat_bot_hex,
            chat_user_hex,
            chat_origin_hex,
            message_id,
            message_indicator_id,
            message_origin_id,
            auto_delete,
        }: InsertTask,
    ) -> Result<i64> {
        let insert_item = tasks::ActiveModel {
            id: ActiveValue::default(),
            cmd_type: Set(cmd_type),
            filename: Set(filename.to_string()),
            root_path: Set(root_path.to_string()),
            url: Set(url),
            upload_url: Set(upload_url.to_string()),
            current_length: Set(current_length as i64),
            total_length: Set(total_length as i64),
            chat_id: Set(chat_id),
            chat_bot_hex: Set(chat_bot_hex.to_string()),
            chat_user_hex: Set(chat_user_hex.to_string()),
            chat_origin_hex: Set(chat_origin_hex),
            message_id: Set(message_id),
            message_indicator_id: Set(message_indicator_id),
            message_origin_id: Set(message_origin_id),
            status: Set(TaskStatus::Waiting),
            auto_delete: Set(auto_delete),
        };

        let id = tasks::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .context("failed to insert url task")?
            .last_insert_id;

        Ok(id)
    }

    pub async fn set_task_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Status, Expr::value(status))
            .exec(&self.connection)
            .await
            .context("failed to update task status")?;

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
            .context("failed to update current length")?;

        Ok(())
    }

    pub async fn get_chats_current_tasks(&self) -> Result<HashMap<ChatHex, Vec<tasks::Model>>> {
        let mut chats = HashMap::new();

        let tasks = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Started))
            .all(&self.connection)
            .await
            .context("failed to get chat current tasks")?;

        for task in tasks {
            chats
                .entry(ChatHex {
                    chat_bot_hex: task.chat_bot_hex.clone(),
                    chat_user_hex: task.chat_user_hex.clone(),
                })
                .or_insert(Vec::new())
                .push(task);
        }

        Ok(chats)
    }

    pub async fn get_chat_pending_tasks_number(&self, chat_bot_hex: &str) -> Result<u64> {
        tasks::Entity::find()
            .filter(
                Condition::all()
                    .add(tasks::Column::ChatBotHex.eq(chat_bot_hex))
                    .add(
                        Condition::any()
                            .add(tasks::Column::Status.eq(TaskStatus::Waiting))
                            .add(tasks::Column::Status.eq(TaskStatus::Fetched)),
                    ),
            )
            .count(&self.connection)
            .await
            .context("failed to get pending tasks number")
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
            .context("failed to get chat started tasks number")?
            > 0;

        Ok(has_started_tasks)
    }

    pub async fn update_filename(&self, id: i64, filename: &str) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Filename, Expr::value(filename))
            .exec(&self.connection)
            .await
            .context("failed to update filename")?;

        Ok(())
    }

    pub async fn delete_task(&self, id: i64) -> Result<()> {
        tasks::Entity::delete_by_id(id)
            .exec(&self.connection)
            .await
            .context("failed to delete task")?;

        Ok(())
    }

    pub async fn clear(&self) -> Result<()> {
        let mut aborters_guard = self.task_aborters.lock().await;
        let aborters = aborters_guard.values();

        for aborter in aborters {
            aborter.abort();
        }

        aborters_guard.clear();

        tasks::Entity::delete_many()
            .exec(&self.connection)
            .await
            .context("failed to clear tasks")?;

        Ok(())
    }

    pub async fn delete_task_from_message_indicator_id_if_exists(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<()> {
        tasks::Entity::delete_many()
            .filter(tasks::Column::ChatId.eq(chat_id))
            .filter(tasks::Column::MessageIndicatorId.eq(message_id))
            .exec(&self.connection)
            .await
            .context("failed to delete task from message indicator id")?;

        Ok(())
    }

    pub async fn delete_task_from_message_id_if_exists(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<()> {
        tasks::Entity::delete_many()
            .filter(tasks::Column::ChatId.eq(chat_id))
            .filter(
                Condition::any()
                    .add(tasks::Column::MessageIndicatorId.eq(message_id))
                    .add(tasks::Column::MessageId.eq(message_id)),
            )
            .exec(&self.connection)
            .await
            .context("failed to delete task from message id or message indicator id")?;

        Ok(())
    }

    pub async fn is_last_task(&self, chat_id: i64, message_indicator_id: i32) -> Result<bool> {
        // check if the task is the last task in batch or /links
        let task = tasks::Entity::find()
            .filter(tasks::Column::ChatId.eq(chat_id))
            .filter(tasks::Column::MessageIndicatorId.eq(message_indicator_id))
            .one(&self.connection)
            .await
            .context("failed to get task with message indicator id")?;

        if let Some(task) = task {
            let count = tasks::Entity::find()
                .filter(tasks::Column::MessageId.eq(task.message_id))
                .count(&self.connection)
                .await
                .context("failed to count with message id")?;

            if count == 1 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn get_message_indicator_ids(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<Vec<i32>> {
        let tasks = tasks::Entity::find()
            .filter(tasks::Column::ChatId.eq(chat_id))
            .filter(tasks::Column::MessageId.eq(message_id))
            .all(&self.connection)
            .await
            .context("failed to get message indicator ids")?;

        Ok(tasks.iter().map(|task| task.message_indicator_id).collect())
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct ChatHex {
    pub chat_bot_hex: String,
    pub chat_user_hex: String,
}

pub struct TaskAborter {
    pub id: i64,
    pub chat_user_hex: String,
    pub message_id: i32,
    filename: String,
    pub token: CancellationToken,
}

impl TaskAborter {
    pub fn new(id: i64, chat_user_hex: &str, message_id: i32, filename: &str) -> Self {
        Self {
            id,
            chat_user_hex: chat_user_hex.to_string(),
            message_id,
            filename: filename.to_string(),
            token: CancellationToken::new(),
        }
    }

    pub fn abort(&self) {
        tracing::info!("task {} aborted", self.filename);

        self.token.cancel();
    }
}

pub struct BatchAborter {
    pub token: CancellationToken,
    // whether the batch is generating command
    pub processing: bool,
}

impl BatchAborter {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            processing: true,
        }
    }

    pub fn abort(&self) {
        tracing::info!("batch or links aborted");

        self.token.cancel();
    }
}
