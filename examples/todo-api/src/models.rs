//! Data models for the todo-api.

use serde::{Deserialize, Serialize};

/// Task stored in the database.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

/// Payload to create a new task.
#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// Payload to update an existing task.
#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}
