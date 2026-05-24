//! Cola asincrona con reintentos y backoff exponencial.
//!
//! Backend en memoria por defecto (Tokio task + canal). Backend opcional
//! persistente via `ag-data` (tabla de jobs) bajo feature
//! `queue-persistent`.
//!
//! Skeleton de la Fase 4.5. Implementacion en la Etapa 2-6.

#[cfg(feature = "queue-persistent")]
pub mod store;
