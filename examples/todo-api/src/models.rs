//! Modelos de datos de la todo-api.

use serde::{Deserialize, Serialize};

/// Tarea almacenada en la base de datos.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub done: bool,
}

/// Payload para crear una tarea nueva.
#[derive(Debug, Deserialize)]
pub struct CreateTodo {
    pub title: String,
}

/// Payload para actualizar una tarea existente.
#[derive(Debug, Deserialize)]
pub struct UpdateTodo {
    pub title: Option<String>,
    pub done: Option<bool>,
}
