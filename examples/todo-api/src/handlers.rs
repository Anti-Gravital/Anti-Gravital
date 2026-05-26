//! HTTP handlers for the todo-api.
//!
//! Each handler receives the necessary extractors and returns
//! `Result<_, AgError>` so that the HTTP response conversion is
//! automatic via `IntoResponse`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;

use ag_core::AgError;

use crate::{
    models::{CreateTodo, Todo, UpdateTodo},
    AppState,
};

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

pub(crate) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "todo-api",
    })
}

pub(crate) async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AgError> {
    let todos = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos ORDER BY id")
        .fetch_all(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?;

    Ok(Json(todos))
}

pub(crate) async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Todo>, AgError> {
    let todo = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?
        .ok_or_else(|| AgError::NotFound(format!("todo {id}")))?;

    Ok(Json(todo))
}

pub(crate) async fn create_todo(
    State(state): State<AppState>,
    Json(req): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), AgError> {
    if req.title.trim().is_empty() {
        return Err(AgError::BadRequest("title cannot be empty".into()));
    }

    let todo = sqlx::query_as::<_, Todo>(
        "INSERT INTO todos (title) VALUES ($1) RETURNING id, title, done",
    )
    .bind(&req.title)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AgError::Database(e.to_string()))?;

    Ok((StatusCode::CREATED, Json(todo)))
}

pub(crate) async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateTodo>,
) -> Result<Json<Todo>, AgError> {
    // Verify it exists before updating.
    let existing = sqlx::query_as::<_, Todo>("SELECT id, title, done FROM todos WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?
        .ok_or_else(|| AgError::NotFound(format!("todo {id}")))?;

    let new_title = req.title.unwrap_or(existing.title);
    let new_done = req.done.unwrap_or(existing.done);

    let updated = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = $1, done = $2 WHERE id = $3 RETURNING id, title, done",
    )
    .bind(&new_title)
    .bind(new_done)
    .bind(id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AgError::Database(e.to_string()))?;

    Ok(Json(updated))
}

pub(crate) async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AgError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| AgError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AgError::NotFound(format!("todo {id}")));
    }

    Ok(StatusCode::NO_CONTENT)
}
