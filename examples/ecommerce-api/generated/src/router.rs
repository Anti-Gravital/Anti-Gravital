//! Router generado por Anti-Gravital ag-dsl v0.2.

use axum::routing;
use super::handlers;

/// Placeholder para el estado de la aplicacion.
pub type AppState = ();

/// Crea el router Axum con todas las rutas definidas en el schema.
pub fn api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/users", routing::get(handlers::list_users))
        .route("/users/:id", routing::get(handlers::get_user))
        .route("/users", routing::post(handlers::create_user))
        .route("/products", routing::get(handlers::list_products))
        .route("/products/:id", routing::get(handlers::get_product))
        .route("/products", routing::post(handlers::create_product))
        .route("/orders", routing::post(handlers::create_order))
        .route("/orders/:id", routing::get(handlers::get_order))
}
