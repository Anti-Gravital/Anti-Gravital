//! Validacion de variables declaradas vs variables usadas en el template.
//!
//! El compilador `ag-dsl` (Etapa 2-8) invocara `check` en build-time para
//! garantizar que las `vars` del bloque `mail` en `schema.ag` coinciden con
//! los placeholders `{{var}}` del HTML del template. Si difieren, el build
//! del proyecto usuario falla con un mensaje claro.
//!
//! Ademas se puede invocar en runtime para validar templates cargados
//! dinamicamente.

use std::collections::HashSet;

use crate::error::AgMailError;

/// Extrae los nombres de variables de un template en formato `{{nombre}}`.
///
/// Soporta espacios alrededor del nombre: `{{ nombre }}` tambien se reconoce.
/// Solo se extrae el nombre — no se valida su tipo.
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

/// Verifica que el template no usa variables fuera del conjunto declarado,
/// y que las variables declaradas aparecen todas en el template.
///
/// - `declared`: variables que el DSL o el caller ha declarado (fuente de verdad).
/// - `template`: string HTML o plaintext del template.
///
/// Retorna `Ok(())` si la interseccion bidireccional es perfecta.
/// Retorna `Err(AgMailError::VarMismatch)` con un mensaje explicativo si no.
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
