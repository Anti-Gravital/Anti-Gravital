//! Capa de validacion de payload.
//!
//! Provee un trait `Validate` que cualquier tipo puede implementar y un
//! extractor `ValidatedJson<T>` que deserializa desde JSON y aplica
//! la validacion automaticamente. Las violaciones se reportan como
//! `AgError::Validation` con detalle estructurado por campo.
//!
//! En Fase 1 la validacion es manual: el desarrollador implementa
//! `Validate` para sus tipos. A partir de Fase 3 el DSL genera tipos
//! que ya implementan `Validate` segun las anotaciones de `.ag`.

use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AgError;

/// Un campo concreto que fallo la validacion.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FieldError {
    /// Nombre del campo afectado en notacion `dot.path`.
    pub field: String,
    /// Mensaje legible para el usuario.
    pub message: String,
}

/// Agregado de errores de validacion para un payload completo.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct ValidationErrors {
    /// Lista de errores por campo. Vacia significa payload valido.
    pub errors: Vec<FieldError>,
}

impl ValidationErrors {
    /// Crea un agregado vacio.
    #[must_use]
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Anade un error por campo.
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError {
            field: field.into(),
            message: message.into(),
        });
    }

    /// Verifica si hay errores.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convierte en `Result` listo para `?`.
    ///
    /// # Errores
    ///
    /// Devuelve `AgError::Validation` si hay al menos un error.
    pub fn into_result(self) -> Result<(), AgError> {
        if self.is_empty() {
            Ok(())
        } else {
            let detail = serde_json::to_string(&self.errors)
                .unwrap_or_else(|_| "<serialization failure>".to_owned());
            Err(AgError::Validation(detail))
        }
    }
}

/// Trait que los tipos validables implementan.
///
/// En Fase 1 los desarrolladores escriben `impl Validate for MiTipo`. En
/// Fase 3 el codegen del DSL genera estas implementaciones desde las
/// anotaciones de `schema.ag`.
pub trait Validate {
    /// Valida el valor y acumula errores en el agregado.
    fn validate(&self, errors: &mut ValidationErrors);
}

/// Extractor JSON con validacion automatica integrada.
///
/// Deserializa el body como JSON al tipo `T` y, si la deserializacion
/// tiene exito, ejecuta `T::validate`. Devuelve `AgError::Validation`
/// con el detalle estructurado si la validacion falla, o el error
/// nativo de Axum si la deserializacion misma falla.
#[derive(Debug, Clone)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate + Send + 'static,
    S: Send + Sync,
{
    type Rejection = AgError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(json_rejection_to_ag_error)?;
        let mut errors = ValidationErrors::new();
        value.validate(&mut errors);
        errors.into_result()?;
        Ok(Self(value))
    }
}

fn json_rejection_to_ag_error(rej: JsonRejection) -> AgError {
    AgError::Validation(rej.body_text())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize)]
    struct CreateUser {
        email: String,
        name: String,
    }

    impl Validate for CreateUser {
        fn validate(&self, errors: &mut ValidationErrors) {
            if !self.email.contains('@') {
                errors.add("email", "must contain @");
            }
            if self.name.is_empty() {
                errors.add("name", "must not be empty");
            }
        }
    }

    #[test]
    fn valid_payload_returns_ok() {
        let user = CreateUser {
            email: "a@b.co".into(),
            name: "Ada".into(),
        };
        let mut errors = ValidationErrors::new();
        user.validate(&mut errors);
        assert!(errors.is_empty());
        assert!(errors.into_result().is_ok());
    }

    #[test]
    fn invalid_payload_collects_errors() {
        let user = CreateUser {
            email: "broken".into(),
            name: String::new(),
        };
        let mut errors = ValidationErrors::new();
        user.validate(&mut errors);
        assert_eq!(errors.errors.len(), 2);
        let result = errors.into_result();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), "validation_error");
    }

    #[test]
    fn field_error_is_serializable() {
        let fe = FieldError {
            field: "email".into(),
            message: "must contain @".into(),
        };
        let json = serde_json::to_string(&fe).unwrap();
        assert!(json.contains("email"));
        assert!(json.contains("must contain @"));
    }

    #[test]
    fn add_appends_in_order() {
        let mut errors = ValidationErrors::new();
        errors.add("a", "one");
        errors.add("b", "two");
        assert_eq!(errors.errors[0].field, "a");
        assert_eq!(errors.errors[1].field, "b");
    }
}
