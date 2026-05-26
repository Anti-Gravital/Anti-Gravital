//! AsyncAPI 2.6 generator from the Anti-DSL AST.
//!
//! Produces the AsyncAPI 2.6 spec in YAML format for all
//! events declared in the schema (v0.6). Returns None if there are no events.

use crate::ast::Schema;
use std::path::PathBuf;

/// Generates the AsyncAPI 2.6 specification as YAML.
/// Returns None if the schema has no declared events.
pub fn generate(schema: &Schema) -> Option<(PathBuf, String)> {
    if schema.events.is_empty() {
        return None;
    }

    let project_name = schema
        .config
        .as_ref()
        .and_then(|c| c.project_name.as_deref())
        .unwrap_or("anti-gravital-app");

    let mut out = String::new();
    out.push_str("asyncapi: '2.6.0'\n");
    out.push_str(&format!(
        "info:\n  title: {}\n  version: '0.1.0'\n",
        project_name
    ));
    out.push_str("channels:\n");

    for ev in &schema.events {
        let channel = ev.name.value.replace('.', "/");
        out.push_str(&format!("  {}:\n", channel));
        out.push_str("    subscribe:\n");
        out.push_str("      message:\n");
        out.push_str(&format!(
            "        payload:\n          $ref: '#/components/schemas/{}'\n",
            ev.payload.value
        ));
        if let Some(days) = ev.retain_days {
            out.push_str(&format!(
                "        bindings:\n          nats:\n            retentionDays: {}\n",
                days
            ));
        }
    }

    out.push_str("components:\n  schemas:\n");
    for ev in &schema.events {
        out.push_str(&format!("    {}:\n      type: object\n", ev.payload.value));
    }

    Some((PathBuf::from("asyncapi.yaml"), out))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{EventDef, Schema, Spanned};

    fn schema_with_event() -> Schema {
        let mut s = Schema::default();
        s.events.push(EventDef {
            name: Spanned::new("user.created".to_string(), 0..12),
            payload: Spanned::new("UserResponse".to_string(), 0..12),
            retain_days: Some(30),
            span: 0..50,
        });
        s
    }

    #[test]
    fn generates_valid_asyncapi_yaml() {
        let schema = schema_with_event();
        let (path, yaml) = generate(&schema).expect("should generate");
        assert_eq!(path.to_str().unwrap(), "asyncapi.yaml");
        assert!(yaml.contains("asyncapi: '2.6.0'"));
        assert!(yaml.contains("user/created"));
        assert!(yaml.contains("UserResponse"));
    }

    #[test]
    fn includes_retention_binding() {
        let schema = schema_with_event();
        let (_, yaml) = generate(&schema).expect("should generate");
        assert!(yaml.contains("retentionDays: 30"));
    }

    #[test]
    fn returns_none_when_no_events() {
        let schema = Schema::default();
        assert!(generate(&schema).is_none());
    }
}
