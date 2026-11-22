//! Compact commerce API derived from the ecommerce DSL example.
//!
//! Default build: in-memory catalog and orders, fully offline.
//! With `--features postgres` and `DATABASE_URL`: ag-data owns the pool and
//! applies `migrations/0001_init.sql`; order creation uses a transaction.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use ag_core::Shield;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
struct Product {
    id: String,
    name: String,
    price_cents: i64,
    stock: i64,
}

#[derive(Clone, Debug, Serialize)]
struct Order {
    id: String,
    product_id: String,
    quantity: i64,
    total_cents: i64,
    status: String,
}

#[derive(Clone)]
enum Store {
    Memory {
        products: Arc<Mutex<HashMap<String, Product>>>,
        orders: Arc<Mutex<HashMap<String, Order>>>,
    },
    #[cfg(feature = "postgres")]
    Postgres(ag_data::DbPool),
}

#[derive(Clone)]
struct AppState {
    store: Store,
    backend: &'static str,
}

#[derive(Debug, Deserialize)]
struct CreateProduct {
    name: String,
    price_cents: i64,
    stock: i64,
}

#[derive(Debug, Deserialize)]
struct CreateOrder {
    product_id: String,
    quantity: i64,
}

#[derive(Debug, Serialize)]
struct Health {
    status: &'static str,
    backend: &'static str,
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: "resource not found".into(),
        }
    }

    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn insufficient_stock() -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "insufficient stock".into(),
        }
    }

    fn internal(err: impl std::fmt::Display) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: err.to_string(),
        }
    }
}

fn lock_error() -> ApiError {
    ApiError::internal("store lock poisoned")
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (self.status, Json(serde_json::json!({ "error": self.message }))).into_response()
    }
}

impl Store {
    async fn list_products(&self) -> Result<Vec<Product>, ApiError> {
        match self {
            Store::Memory { products, .. } => {
                let guard = products.lock().map_err(|_| lock_error())?;
                let mut items: Vec<Product> = guard.values().cloned().collect();
                items.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(items)
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                use sqlx::Row;
                let rows = sqlx::query("SELECT id, name, price_cents, stock FROM products ORDER BY name")
                    .fetch_all(pool)
                    .await
                    .map_err(ApiError::internal)?;
                rows.into_iter()
                    .map(|row| {
                        Ok(Product {
                            id: row.try_get("id").map_err(ApiError::internal)?,
                            name: row.try_get("name").map_err(ApiError::internal)?,
                            price_cents: row.try_get("price_cents").map_err(ApiError::internal)?,
                            stock: row.try_get("stock").map_err(ApiError::internal)?,
                        })
                    })
                    .collect()
            }
        }
    }

    async fn create_product(&self, name: String, price_cents: i64, stock: i64) -> Result<Product, ApiError> {
        if name.trim().is_empty() || price_cents < 0 || stock < 0 {
            return Err(ApiError::bad_request("invalid product fields"));
        }
        let product = Product {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            price_cents,
            stock,
        };
        match self {
            Store::Memory { products, .. } => {
                products
                    .lock()
                    .map_err(|_| lock_error())?
                    .insert(product.id.clone(), product.clone());
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                sqlx::query("INSERT INTO products (id, name, price_cents, stock) VALUES ($1, $2, $3, $4)")
                    .bind(&product.id)
                    .bind(&product.name)
                    .bind(product.price_cents)
                    .bind(product.stock)
                    .execute(pool)
                    .await
                    .map_err(ApiError::internal)?;
            }
        }
        Ok(product)
    }

    async fn create_order(&self, product_id: &str, quantity: i64) -> Result<Order, ApiError> {
        if quantity <= 0 {
            return Err(ApiError::bad_request("quantity must be positive"));
        }
        match self {
            Store::Memory { products, orders } => {
                let mut products = products.lock().map_err(|_| lock_error())?;
                let product = products
                    .get_mut(product_id)
                    .ok_or_else(ApiError::not_found)?;
                if product.stock < quantity {
                    return Err(ApiError::insufficient_stock());
                }
                product.stock -= quantity;
                let order = Order {
                    id: uuid::Uuid::new_v4().to_string(),
                    product_id: product_id.to_owned(),
                    quantity,
                    total_cents: product.price_cents * quantity,
                    status: "confirmed".into(),
                };
                orders
                    .lock()
                    .map_err(|_| lock_error())?
                    .insert(order.id.clone(), order.clone());
                Ok(order)
            }
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                use sqlx::Row;
                let mut tx = ag_data::AgTx::begin(pool).await.map_err(ApiError::internal)?;
                let row = sqlx::query("SELECT price_cents, stock FROM products WHERE id = $1 FOR UPDATE")
                    .bind(product_id)
                    .fetch_optional(tx.as_executor())
                    .await
                    .map_err(ApiError::internal)?
                    .ok_or_else(ApiError::not_found)?;
                let price_cents: i64 = row.try_get("price_cents").map_err(ApiError::internal)?;
                let stock: i64 = row.try_get("stock").map_err(ApiError::internal)?;
                if stock < quantity {
                    return Err(ApiError::insufficient_stock());
                }
                sqlx::query("UPDATE products SET stock = $1 WHERE id = $2")
                    .bind(stock - quantity)
                    .bind(product_id)
                    .execute(tx.as_executor())
                    .await
                    .map_err(ApiError::internal)?;
                let order = Order {
                    id: uuid::Uuid::new_v4().to_string(),
                    product_id: product_id.to_owned(),
                    quantity,
                    total_cents: price_cents * quantity,
                    status: "confirmed".into(),
                };
                sqlx::query("INSERT INTO orders (id, product_id, quantity, total_cents, status) VALUES ($1, $2, $3, $4, $5)")
                    .bind(&order.id)
                    .bind(&order.product_id)
                    .bind(order.quantity)
                    .bind(order.total_cents)
                    .bind(&order.status)
                    .execute(tx.as_executor())
                    .await
                    .map_err(ApiError::internal)?;
                tx.commit().await.map_err(ApiError::internal)?;
                Ok(order)
            }
        }
    }

    async fn get_order(&self, id: &str) -> Result<Order, ApiError> {
        match self {
            Store::Memory { orders, .. } => orders
                .lock()
                .map_err(|_| lock_error())?
                .get(id)
                .cloned()
                .ok_or_else(ApiError::not_found),
            #[cfg(feature = "postgres")]
            Store::Postgres(pool) => {
                use sqlx::Row;
                let row = sqlx::query("SELECT id, product_id, quantity, total_cents, status FROM orders WHERE id = $1")
                    .bind(id)
                    .fetch_optional(pool)
                    .await
                    .map_err(ApiError::internal)?
                    .ok_or_else(ApiError::not_found)?;
                Ok(Order {
                    id: row.try_get("id").map_err(ApiError::internal)?,
                    product_id: row.try_get("product_id").map_err(ApiError::internal)?,
                    quantity: row.try_get("quantity").map_err(ApiError::internal)?,
                    total_cents: row.try_get("total_cents").map_err(ApiError::internal)?,
                    status: row.try_get("status").map_err(ApiError::internal)?,
                })
            }
        }
    }
}

async fn build_store() -> Result<(Store, &'static str), Box<dyn std::error::Error>> {
    #[cfg(feature = "postgres")]
    if let Ok(url) = std::env::var("DATABASE_URL") {
        let config = ag_data::DataConfig {
            url,
            ..ag_data::DataConfig::default()
        };
        let pool = ag_data::connect(&config).await?;
        static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
        ag_data::run_migrations(&pool, &MIGRATOR).await?;
        return Ok((Store::Postgres(pool), "postgres"));
    }

    #[cfg(not(feature = "postgres"))]
    if std::env::var("DATABASE_URL").is_ok() {
        tracing::warn!("DATABASE_URL set but the postgres feature is off; using in-memory store");
    }

    let mut products = HashMap::new();
    for (name, price_cents, stock) in [
        ("notebook", 499, 10),
        ("pencil", 25, 100),
        ("eraser", 15, 50),
    ] {
        let product = Product {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            price_cents,
            stock,
        };
        products.insert(product.id.clone(), product);
    }
    Ok((
        Store::Memory {
            products: Arc::new(Mutex::new(products)),
            orders: Arc::new(Mutex::new(HashMap::new())),
        },
        "memory",
    ))
}

async fn list_products_handler(State(state): State<AppState>) -> Result<Json<Vec<Product>>, ApiError> {
    Ok(Json(state.store.list_products().await?))
}

async fn create_product_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateProduct>,
) -> Result<(StatusCode, Json<Product>), ApiError> {
    let product = state
        .store
        .create_product(payload.name, payload.price_cents, payload.stock)
        .await?;
    Ok((StatusCode::CREATED, Json(product)))
}

async fn create_order_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateOrder>,
) -> Result<(StatusCode, Json<Order>), ApiError> {
    let order = state
        .store
        .create_order(&payload.product_id, payload.quantity)
        .await?;
    Ok((StatusCode::CREATED, Json(order)))
}

async fn get_order_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Order>, ApiError> {
    Ok(Json(state.store.get_order(&id).await?))
}

async fn health_handler(State(state): State<AppState>) -> Json<Health> {
    Json(Health {
        status: "ok",
        backend: state.backend,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let (store, backend) = build_store().await?;
    tracing::info!(backend, "commerce store ready");
    let state = AppState { store, backend };

    let shield = Shield::with_defaults();
    let router = Router::new()
        .route("/products", get(list_products_handler).post(create_product_handler))
        .route("/orders", axum::routing::post(create_order_handler))
        .route("/orders/:id", get(get_order_handler))
        .route("/health", get(health_handler))
        .with_state(state);

    let app = shield.apply(router);
    let bind = std::env::var("BIND").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(&bind).await?;
    tracing::info!(%bind, "listening");
    shield.serve(listener, app).await?;
    Ok(())
}
