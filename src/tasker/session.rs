/*
:project: telegram-onedrive
:author: L-ING
:copyright: (C) 2024 L-ING <hlf01@icloud.com>
:license: MIT, see LICENSE for more details.
*/

use super::tasks::{self, InsertTask, TaskStatus};
use crate::error::{Error, Result};
use proc_macros::{add_context, add_trace};
use sea_orm::{
    sea_query::Expr, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DatabaseConnection,
    EntityName, EntityTrait, PaginatorTrait, QueryFilter, Schema, Set,
};
use std::{collections::HashMap, path::Path, sync::Arc};
use tokio::{fs, sync::Mutex};
use tokio_util::sync::CancellationToken;

// (chat id, message id) -> (task aborter, related message id if exists)
pub type TaskAborters = Arc<Mutex<HashMap<(i64, i32), (Arc<TaskAborter>, Option<i32>)>>>;

pub struct TaskSession {
    connection: DatabaseConnection,
    pub aborters: TaskAborters,
}

impl TaskSession {
    #[add_context]
    pub async fn new(session_path: &str) -> Result<Self> {
        if Path::new(session_path).exists() {
            fs::remove_file(session_path)
                .await
                .map_err(|e| Error::new("failed to remove old task session").raw(e))?;
        }

        let connection = Self::connect_db(session_path).await?;
        let aborters = Arc::new(Mutex::new(HashMap::new()));

        Ok(Self {
            connection,
            aborters,
        })
    }

    #[add_context]
    #[add_trace]
    async fn connect_db(path: &str) -> Result<DatabaseConnection> {
        let connection = sea_orm::Database::connect(format!("sqlite://{}?mode=rwc", path))
            .await
            .map_err(|e| Error::new("failed to connect to task session").raw(e))?;

        Self::create_table_if_not_exists(&connection).await?;

        Ok(connection)
    }

    #[add_context]
    #[add_trace]
    async fn create_table_if_not_exists(connection: &DatabaseConnection) -> Result<()> {
        if !Self::is_table_exists(connection).await {
            let backend = connection.get_database_backend();

            let table_create_statement =
                Schema::new(backend).create_table_from_entity(tasks::Entity);

            connection
                .execute(backend.build(&table_create_statement))
                .await
                .map_err(|e| {
                    Error::new(format!(
                        "failed to create table {}",
                        tasks::Entity.table_name()
                    ))
                    .raw(e)
                })?;
        }

        Ok(())
    }

    #[add_trace]
    async fn is_table_exists(connection: &DatabaseConnection) -> bool {
        let result = tasks::Entity::find().all(connection).await;

        result.is_ok()
    }

    #[add_context]
    pub async fn fetch_task(&self) -> Result<Option<tasks::Model>> {
        let task = tasks::Entity::find()
            .filter(tasks::Column::Status.eq(TaskStatus::Waiting))
            .one(&self.connection)
            .await
            .map_err(|e| Error::new("failed to get a task").raw(e))?;

        if let Some(task) = &task {
            self.set_task_status(task.id, tasks::TaskStatus::Fetched)
                .await?;
        }

        Ok(task)
    }

    #[add_context]
    #[add_trace]
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
            message_id_forward,
            message_id_origin,
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
            message_id_forward: Set(message_id_forward),
            message_id_origin: Set(message_id_origin),
            status: Set(TaskStatus::Waiting),
            auto_delete: Set(auto_delete),
        };

        let id = tasks::Entity::insert(insert_item)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to insert url task").raw(e))?
            .last_insert_id;

        Ok(id)
    }

    #[add_context]
    #[add_trace]
    pub async fn set_task_status(&self, id: i64, status: TaskStatus) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Status, Expr::value(status))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to update task status").raw(e))?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn set_current_length(&self, id: i64, current_length: u64) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(
                tasks::Column::CurrentLength,
                Expr::value(current_length as i64),
            )
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to update current length").raw(e))?;

        Ok(())
    }

    #[add_context]
    pub async fn get_chats_tasks(&self) -> Result<HashMap<ChatHex, ChatTasks>> {
        let mut chats = HashMap::new();

        macro_rules! insert_chat_tasks {
            ($status: ident, $task_type: ident) => {
                let tasks = tasks::Entity::find()
                    .filter(tasks::Column::Status.eq(TaskStatus::$status))
                    .all(&self.connection)
                    .await
                    .map_err(|e| Error::new("failed to get chat current tasks").raw(e))?;

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

    #[add_context]
    #[add_trace]
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
            .map_err(|e| Error::new("failed to get pending tasks number").raw(e))
    }

    #[add_context]
    #[add_trace]
    pub async fn does_chat_has_started_tasks(&self, chat_bot_hex: &str) -> Result<bool> {
        let has_started_tasks = tasks::Entity::find()
            .filter(
                Condition::all()
                    .add(tasks::Column::ChatBotHex.eq(chat_bot_hex))
                    .add(tasks::Column::Status.eq(TaskStatus::Started)),
            )
            .count(&self.connection)
            .await
            .map_err(|e| Error::new("failed to get chat started tasks number").raw(e))?
            > 0;

        Ok(has_started_tasks)
    }

    #[add_context]
    #[add_trace]
    pub async fn update_filename(&self, id: i64, filename: &str) -> Result<()> {
        tasks::Entity::update_many()
            .filter(tasks::Column::Id.eq(id))
            .col_expr(tasks::Column::Filename, Expr::value(filename))
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to update filename").raw(e))?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn delete_task(&self, id: i64) -> Result<()> {
        tasks::Entity::delete_by_id(id)
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to delete task").raw(e))?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn clear(&self) -> Result<()> {
        let mut aborters_guard = self.aborters.lock().await;
        let aborters = aborters_guard.values();

        for (aborter, _) in aborters {
            aborter.abort();
        }

        aborters_guard.clear();

        tasks::Entity::delete_many()
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to clear tasks").raw(e))?;

        Ok(())
    }

    #[add_context]
    #[add_trace]
    pub async fn delete_task_from_message_id_if_exists(
        &self,
        chat_id: i64,
        message_id: i32,
    ) -> Result<()> {
        tasks::Entity::delete_many()
            .filter(tasks::Column::ChatId.eq(chat_id))
            .filter(
                Condition::any()
                    .add(tasks::Column::MessageId.eq(message_id))
                    .add(tasks::Column::MessageIdForward.eq(Some(message_id))),
            )
            .exec(&self.connection)
            .await
            .map_err(|e| Error::new("failed to delete task from message id").raw(e))?;

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

pub struct TaskAborter {
    pub id: i64,
    filename: String,
    pub token: CancellationToken,
}

impl TaskAborter {
    pub fn new(id: i64, filename: &str) -> Self {
        Self {
            id,
            filename: filename.to_string(),
            token: CancellationToken::new(),
        }
    }

    pub fn abort(&self) {
        tracing::info!("task {} aborted", self.filename);

        self.token.cancel();
    }
}
