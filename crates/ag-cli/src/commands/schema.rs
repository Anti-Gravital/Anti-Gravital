//! DSL commands: `ag generate`, `ag schema lint`, `ag schema diff`.

use super::project::write_file;
use crate::output::{DiagnosticView, SchemaChangeView, SchemaDiffData, SchemaLintData};
use std::fs;
use std::path::Path;

pub(crate) fn cmd_generate(schema_path: &Path, output_dir: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;

    let schema = ag_dsl::compile(&source).map_err(|diags| {
        let mut msg = format!(
            "{} error(s) in '{}':\n",
            diags.iter().filter(|d| d.is_error()).count(),
            schema_path.display()
        );
        for d in &diags {
            msg.push_str(&format!("  {}\n", d.display(&source)));
        }
        msg
    })?;

    let files = ag_dsl::generate(&schema);

    println!(
        "Generating {} artifacts from '{}'...",
        files.len(),
        schema_path.display()
    );

    for (rel_path, content) in &files.files {
        let full_path = output_dir.join(rel_path);
        write_file(&full_path, content)?;
        println!("  {}", full_path.display());
    }

    println!("Generation completed in '{}'.", output_dir.display());
    Ok(())
}

pub(crate) fn cmd_schema_lint(schema_path: &Path) -> Result<(), String> {
    let source = read_schema(schema_path)?;
    let diags = ag_dsl::lint(&source);

    if diags.is_empty() {
        println!("'{}': no issues found.", schema_path.display());
        return Ok(());
    }

    let errors: Vec<_> = diags.iter().filter(|d| d.is_error()).collect();
    let warnings: Vec<_> = diags.iter().filter(|d| !d.is_error()).collect();

    for w in &warnings {
        println!("warning: {}", w.display(&source));
    }
    for e in &errors {
        eprintln!("error:   {}", e.display(&source));
    }

    if !errors.is_empty() {
        return Err(format!(
            "{} error(s) in '{}'",
            errors.len(),
            schema_path.display()
        ));
    }
    Ok(())
}

/// Builds the versioned `schema.lint` result without printing prose.
/// Validation errors are reported in the data, not as a Rust error, so a
/// machine consumer always receives a complete versioned document.
pub(crate) fn lint_report(schema_path: &Path) -> Result<SchemaLintData, String> {
    let source = read_schema(schema_path)?;
    let diags = ag_dsl::lint(&source);
    let error_count = diags.iter().filter(|d| d.is_error()).count();
    let warning_count = diags.len() - error_count;
    let diagnostics = diags
        .iter()
        .map(|d| DiagnosticView {
            severity: if d.is_error() { "error" } else { "warning" },
            message: d.message.clone(),
            hint: d.hint.clone(),
            span_start: d.span.start,
            span_end: d.span.end,
        })
        .collect();
    let result = if error_count > 0 {
        "errors"
    } else if warning_count > 0 {
        "warnings"
    } else {
        "clean"
    };
    Ok(SchemaLintData {
        schema: schema_path.display().to_string(),
        result,
        error_count,
        warning_count,
        diagnostics,
    })
}

/// Compiles one side of a diff, preserving the historical prose error text.
fn compile_diff_schema(source: &str, label: &str) -> Result<ag_dsl::ast::Schema, String> {
    ag_dsl::compile(source).map_err(|diags| {
        format!(
            "errors in {label} schema: {}",
            diags
                .iter()
                .filter(|d| d.is_error())
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })
}

/// Builds the versioned `schema.diff` result without printing prose.
pub(crate) fn diff_report(
    schema_path: &Path,
    reference_path: &Path,
) -> Result<SchemaDiffData, String> {
    let current_source = read_schema(schema_path)?;
    let ref_source = read_schema(reference_path)?;
    let current = compile_diff_schema(&current_source, "current")?;
    let reference = compile_diff_schema(&ref_source, "reference")?;
    let changes = diff_changes(&reference, &current)
        .into_iter()
        .map(|change| SchemaChangeView {
            severity: if change.breaking {
                "breaking"
            } else {
                "additive"
            },
            message: change.message,
        })
        .collect();
    Ok(SchemaDiffData {
        reference: reference_path.display().to_string(),
        schema: schema_path.display().to_string(),
        changes,
    })
}

pub(crate) fn cmd_schema_diff(schema_path: &Path, reference_path: &Path) -> Result<(), String> {
    let current_source = read_schema(schema_path)?;
    let ref_source = read_schema(reference_path)?;

    let current = compile_diff_schema(&current_source, "current")?;
    let reference = compile_diff_schema(&ref_source, "reference")?;

    let changes = diff_schemas(&reference, &current);

    if changes.is_empty() {
        println!(
            "No changes between '{}' and '{}'.",
            reference_path.display(),
            schema_path.display()
        );
        return Ok(());
    }

    println!(
        "Changes between '{}' and '{}':",
        reference_path.display(),
        schema_path.display()
    );
    for change in &changes {
        println!("  {change}");
    }
    Ok(())
}

/// A single structured schema change, shared by prose and JSON output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SchemaChange {
    pub(crate) breaking: bool,
    pub(crate) message: String,
}

impl SchemaChange {
    fn breaking(message: impl Into<String>) -> Self {
        Self {
            breaking: true,
            message: message.into(),
        }
    }

    fn additive(message: impl Into<String>) -> Self {
        Self {
            breaking: false,
            message: message.into(),
        }
    }
}

/// Compares two schemas and returns structured changes.
///
/// Detects: added/removed models, added/removed/changed fields.
/// Removing a model, removing a field or changing a type is breaking;
/// additions are non-breaking.
pub(crate) fn diff_changes(
    old: &ag_dsl::ast::Schema,
    new: &ag_dsl::ast::Schema,
) -> Vec<SchemaChange> {
    use std::collections::HashMap;

    let old_models: HashMap<&str, _> = old
        .models
        .iter()
        .map(|m| (m.name.value.as_str(), m))
        .collect();
    let new_models: HashMap<&str, _> = new
        .models
        .iter()
        .map(|m| (m.name.value.as_str(), m))
        .collect();

    let mut changes = Vec::new();

    // Removed models (breaking)
    for name in old_models.keys() {
        if !new_models.contains_key(name) {
            changes.push(SchemaChange::breaking(format!("model '{name}' removed")));
        }
    }

    // Added models (non-breaking)
    for name in new_models.keys() {
        if !old_models.contains_key(name) {
            changes.push(SchemaChange::additive(format!("model '{name}' added")));
        }
    }

    // Fields in common models
    for (name, old_model) in &old_models {
        if let Some(new_model) = new_models.get(name) {
            let old_fields: HashMap<&str, _> = old_model
                .fields
                .iter()
                .map(|f| (f.name.value.as_str(), f))
                .collect();
            let new_fields: HashMap<&str, _> = new_model
                .fields
                .iter()
                .map(|f| (f.name.value.as_str(), f))
                .collect();

            for fname in old_fields.keys() {
                if !new_fields.contains_key(fname) {
                    changes.push(SchemaChange::breaking(format!("'{name}.{fname}' removed")));
                }
            }
            for fname in new_fields.keys() {
                if !old_fields.contains_key(fname) {
                    changes.push(SchemaChange::additive(format!("'{name}.{fname}' added")));
                }
            }
            for (fname, old_field) in &old_fields {
                if let Some(new_field) = new_fields.get(fname) {
                    if old_field.ty.value != new_field.ty.value {
                        changes.push(SchemaChange::breaking(format!(
                            "'{name}.{fname}' type changed: {:?} -> {:?}",
                            old_field.ty.value, new_field.ty.value
                        )));
                    }
                    if old_field.optional && !new_field.optional {
                        changes.push(SchemaChange::breaking(format!(
                            "'{name}.{fname}' changed from nullable to NOT NULL"
                        )));
                    }
                    if !old_field.optional && new_field.optional {
                        changes.push(SchemaChange::additive(format!(
                            "'{name}.{fname}' changed from NOT NULL to nullable"
                        )));
                    }
                }
            }
        }
    }

    changes
}

/// Compares two schemas and returns a list of readable changes.
///
/// This is the prose front-end; JSON consumers use [`diff_changes`] through
/// [`diff_report`]. The prefixes are added here so both shapes stay in sync.
pub(crate) fn diff_schemas(old: &ag_dsl::ast::Schema, new: &ag_dsl::ast::Schema) -> Vec<String> {
    diff_changes(old, new)
        .into_iter()
        .map(|change| {
            let prefix = if change.breaking {
                "[BREAKING]  "
            } else {
                "[additive]  "
            };
            format!("{prefix}{}", change.message)
        })
        .collect()
}

pub(crate) fn read_schema(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("schema file not found: '{}'", path.display()));
    }
    fs::read_to_string(path).map_err(|e| format!("failed to read '{}': {e}", path.display()))
}
