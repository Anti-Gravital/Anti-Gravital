//! Payload validation layer.
//!
//! Provides a `Validate` trait that any type can implement and a
//! `ValidatedJson<T>` extractor that deserializes from JSON and applies
//! validation automatically. Violations are reported as
//! `AgError::Validation` with structured per-field detail.
//!
//! In Phase 1 validation is manual: the developer implements `Validate`
//! for their types. Starting in Phase 3 the DSL generates types that
//! already implement `Validate` from the `.ag` annotations.

use axum::async_trait;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use axum::Json;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AgError;

/// A specific field that failed validation.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FieldError {
    /// Name of the affected field in `dot.path` notation.
    pub field: String,
    /// User-readable message.
    pub message: String,
}

/// Aggregate of validation errors for a complete payload.
#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq)]
pub struct ValidationErrors {
    /// List of per-field errors. Empty means a valid payload.
    pub errors: Vec<FieldError>,
}

impl ValidationErrors {
    /// Creates an empty aggregate.
    #[must_use]
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    /// Adds a per-field error.
    pub fn add(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(FieldError {
            field: field.into(),
            message: message.into(),
        });
    }

    /// Checks whether there are any errors.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Converts into a `Result` ready for `?`.
    ///
    /// # Errors
    ///
    /// Returns `AgError::Validation` if there is at least one error.
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

/// Trait that validatable types implement.
///
/// In Phase 1 developers write `impl Validate for MiTipo`. In Phase 3
/// the DSL codegen generates these implementations from the
/// `schema.ag` annotations.
pub trait Validate {
    /// Validates the value and accumulates errors in the aggregate.
    fn validate(&self, errors: &mut ValidationErrors);
}

/// JSON extractor with built-in automatic validation.
///
/// Deserializes the body as JSON into the type `T` and, if
/// deserialization succeeds, runs `T::validate`. Returns
/// `AgError::Validation` with the structured detail if validation fails,
/// or the native Axum error if deserialization itself fails.
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
