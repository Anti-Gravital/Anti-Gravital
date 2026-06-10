//! Background worker code generator (DSL v0.8, ag-workers / RFC-0012).
//!
//! For each `worker` declaration this emits a typed payload struct (from the `input`
//! block), a `JobHandler` implementation stub the developer fills in, and a
//! `register_workers` function that wires every handler into a closed registry. The
//! generated file is only produced when the schema declares at least one worker.

use std::path::PathBuf;

use crate::ast::{Schema, WorkerDef};

/// Generates `src/workers.rs` when the schema declares workers; otherwise `None`.
pub fn generate(schema: &Schema) -> Option<(PathBuf, String)> {
    if schema.workers.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str(
        "//! Generated worker payloads and handler stubs (ag-workers, RFC-0012).\n\
         //!\n\
         //! Payload structs are generated from the `input` block of each `worker`\n\
         //! declaration; do not edit them. Implement the `handle` bodies of the\n\
         //! generated handler stubs. Register the handlers with `register_workers`.\n\n\
         use ag_workers::{JobContext, JobHandler, JobOutcome, JobPayload, WorkerError};\n\
         use serde::{Deserialize, Serialize};\n\n",
    );

    for w in &schema.workers {
        out.push_str(&generate_worker(w));
    }

    out.push_str(&generate_registry(schema));

    Some((PathBuf::from("src/workers.rs"), out))
}

fn payload_type(name: &str) -> String {
    format!("{name}Payload")
}

fn handler_type(name: &str) -> String {
    format!("{name}Handler")
}

fn generate_worker(w: &WorkerDef) -> String {
    let name = &w.name.value;
    let payload = payload_type(name);
    let handler = handler_type(name);

    let mut s = String::new();

    // Doc comment summarizing the declared queue/mode.
    let queue = w.queue.as_deref().unwrap_or("default");
    s.push_str(&format!(
        "/// Payload for the `{name}` worker (queue `{queue}`).\n"
    ));
    s.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    s.push_str(&format!("pub struct {payload} {{\n"));
    for field in &w.input {
        if field.virtual_field {
            continue; // relations are not part of a job payload
        }
        let fname = &field.name.value;
        let fty = field.ty.value.rust_type(field.optional);
        s.push_str(&format!("    /// `{fname}` input field.\n"));
        s.push_str(&format!("    pub {fname}: {fty},\n"));
    }
    s.push_str("}\n\n");

    s.push_str(&format!(
        "impl JobPayload for {payload} {{\n    const PAYLOAD_VERSION: u16 = 1;\n}}\n\n"
    ));

    s.push_str(&format!(
        "/// Handler stub for the `{name}` worker. Implement `handle`.\n\
         pub struct {handler};\n\n\
         #[async_trait::async_trait]\n\
         impl JobHandler for {handler} {{\n\
         \x20   type Payload = {payload};\n\
         \x20   fn kind(&self) -> &'static str {{\n        \"{name}\"\n    }}\n\
         \x20   async fn handle(\n\
         \x20       &self,\n\
         \x20       _ctx: JobContext,\n\
         \x20       _payload: Self::Payload,\n\
         \x20   ) -> Result<JobOutcome, WorkerError> {{\n\
         \x20       // TODO: implement the `{name}` job logic.\n\
         \x20       Ok(JobOutcome::Complete)\n\
         \x20   }}\n\
         }}\n\n"
    ));

    s
}

fn generate_registry(schema: &Schema) -> String {
    let mut s = String::new();
    s.push_str(
        "/// Registers every generated worker handler into a registry builder.\n\
         pub fn register_workers(\n\
         \x20   builder: ag_workers::WorkerRegistryBuilder,\n\
         ) -> Result<ag_workers::WorkerRegistryBuilder, ag_workers::RegistryError> {\n",
    );
    if schema.workers.is_empty() {
        s.push_str("    Ok(builder)\n}\n");
        return s;
    }
    s.push_str("    let builder = builder\n");
    for (i, w) in schema.workers.iter().enumerate() {
        let handler = handler_type(&w.name.value);
        let tail = if i + 1 == schema.workers.len() {
            ";"
        } else {
            ""
        };
        s.push_str(&format!("        .register({handler})?{tail}\n"));
    }
    s.push_str("    Ok(builder)\n}\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;
    use crate::parser::parse_tokens;

    fn schema_from(src: &str) -> Schema {
        let (tokens, _) = tokenize(src);
        let (ast, diags) = parse_tokens(tokens, src.len());
        assert!(
            diags.iter().all(|d| !d.is_error()),
            "parse errors: {diags:?}"
        );
        ast.expect("valid schema")
    }

    #[test]
    fn no_workers_no_file() {
        let schema = schema_from("model User { id UUID @primary @auto }");
        assert!(generate(&schema).is_none());
    }

    #[test]
    fn generates_payload_handler_and_registry() {
        let src = r#"
worker SendWelcomeEmail {
    queue "mail"
    mode static
    concurrency 4
    timeout "30s"
    retry {
        max_attempts 5
        backoff exponential
        jitter true
    }
    input {
        user_id UUID
        email String @email
    }
    emits "mail.sent"
    on_failure dlq
}
"#;
        let schema = schema_from(src);
        let (path, content) = generate(&schema).expect("should generate");
        assert_eq!(path.to_str().unwrap(), "src/workers.rs");
        assert!(content.contains("pub struct SendWelcomeEmailPayload"));
        assert!(content.contains("pub user_id: uuid::Uuid"));
        assert!(content.contains("pub email: String"));
        assert!(content.contains("impl JobPayload for SendWelcomeEmailPayload"));
        assert!(content.contains("impl JobHandler for SendWelcomeEmailHandler"));
        assert!(content.contains("\"SendWelcomeEmail\""));
        assert!(content.contains(".register(SendWelcomeEmailHandler)?;"));
    }
}
