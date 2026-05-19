//! Capa Core: router, extractores tipados y estado compartido.
//!
//! Fase 1 deja este modulo como placeholder estable: solo expone el
//! tipo `AppState` minimo. La implementacion completa (router con
//! extractores `State<T>`, `ValidatedBody<T>`, `Claims<T>`, `Path<T>`,
//! `Query<T>`, sistema de respuestas JSON/plaintext/streams) llega en
//! Fase 2 segun la Hoja de Ruta.
//!
//! La separacion logica Shield/Core descrita en el capitulo 6 del
//! maestro de arquitectura se respeta desde el bootstrap.

/// Estado compartido por defecto. Los proyectos lo extienden con sus
/// clientes a base de datos, caches y servicios externos.
#[derive(Debug, Default, Clone)]
pub struct AppState;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_default_is_constructible() {
        let _state = AppState;
    }
}
