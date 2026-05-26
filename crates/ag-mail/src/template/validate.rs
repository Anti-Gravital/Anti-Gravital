//! Validation of declared variables vs variables used in the template.
//!
//! The `ag-dsl` compiler (Stage 2-8) will invoke `check` at build-time to
//! ensure that the `vars` of the `mail` block in `schema.ag` match the
//! `{{var}}` placeholders in the template HTML. If they differ, the user
//! project build fails with a clear message.
//!
//! It can also be invoked at runtime to validate dynamically loaded
//! templates.

use std::collections::HashSet;

use crate::error::AgMailError;

/// Extracts the variable names from a template in `{{nombre}}` format.
///
/// Supports spaces around the name: `{{ nombre }}` is also recognized.
/// Only the name is extracted — its type is not validated.
pub fn extract_vars(template: &str) -> HashSet<String> {
    let mut vars = HashSet::new();
    let mut rest = template;

    while let Some(open) = rest.find("{{") {
        rest = &rest[open + 2..];
        if let Some(close) = rest.find("}}") {
            let name = rest[..close].trim().to_owned();
            if !name.is_empty() {
                vars.insert(name);
            }
            rest = &rest[close + 2..];
        } else {
            break;
        }
    }
    vars
}

/// Verifies that the template does not use variables outside the declared set,
/// and that all declared variables appear in the template.
///
/// - `declared`: variables that the DSL or the caller has declared (source of truth).
/// - `template`: HTML or plaintext string of the template.
///
/// Returns `Ok(())` if the bidirectional intersection is perfect.
/// Returns `Err(AgMailError::VarMismatch)` with an explanatory message otherwise.
pub fn check(declared: &HashSet<String>, template: &str) -> Result<(), AgMailError> {
    let used = extract_vars(template);

    let undeclared: Vec<&str> = used
        .iter()
        .filter(|v| !declared.contains(*v))
        .map(|s| s.as_str())
        .collect();

    let unused: Vec<&str> = declared
        .iter()
        .filter(|v| !used.contains(*v))
        .map(|s| s.as_str())
        .collect();

    if undeclared.is_empty() && unused.is_empty() {
        return Ok(());
    }

    let mut parts = Vec::new();
    if !undeclared.is_empty() {
        let mut sorted = undeclared.clone();
        sorted.sort_unstable();
        parts.push(format!("usadas sin declarar: [{}]", sorted.join(", ")));
    }
    if !unused.is_empty() {
        let mut sorted = unused.clone();
        sorted.sort_unstable();
        parts.push(format!("declaradas sin usar: [{}]", sorted.join(", ")));
    }

    Err(AgMailError::VarMismatch(parts.join("; ")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_simple_vars() {
        let vars = extract_vars("Hola {{nombre}}, tu token es {{token}}.");
        assert_eq!(vars.len(), 2);
        assert!(vars.contains("nombre"));
        assert!(vars.contains("token"));
    }

    #[test]
    fn extract_vars_with_spaces() {
        let vars = extract_vars("{{ nombre }} y {{ apellido }}");
        assert!(vars.contains("nombre"));
        assert!(vars.contains("apellido"));
    }

    #[test]
    fn extract_no_vars() {
        let vars = extract_vars("Texto sin variables");
        assert!(vars.is_empty());
    }

    #[test]
    fn check_ok_when_matching() {
        let declared: HashSet<String> = ["nombre".to_owned(), "token".to_owned()]
            .into_iter()
            .collect();
        let tpl = "Hola {{nombre}}, tu token es {{token}}.";
        assert!(check(&declared, tpl).is_ok());
    }

    #[test]
    fn check_fails_undeclared_var() {
        let declared: HashSet<String> = ["nombre".to_owned()].into_iter().collect();
        let tpl = "Hola {{nombre}}, clave: {{clave_secreta}}.";
        let err = check(&declared, tpl).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("clave_secreta"),
            "debe mencionar la var no declarada"
        );
    }

    #[test]
    fn check_fails_unused_declared_var() {
        let declared: HashSet<String> = ["nombre".to_owned(), "apellido".to_owned()]
            .into_iter()
            .collect();
        let tpl = "Hola {{nombre}}.";
        let err = check(&declared, tpl).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("apellido"),
            "debe mencionar la var declarada sin usar"
        );
    }

    #[test]
    fn check_empty_template_and_declared() {
        let declared: HashSet<String> = HashSet::new();
        assert!(check(&declared, "").is_ok());
        assert!(check(&declared, "texto sin vars").is_ok());
    }
}
