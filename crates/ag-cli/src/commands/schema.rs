//! DSL commands: `ag generate`, `ag schema lint`, `ag schema diff`.

use super::project::write_file;
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

pub(crate) fn cmd_schema_diff(schema_path: &Path, reference_path: &Path) -> Result<(), String> {
    let current_source = read_schema(schema_path)?;
    let ref_source = read_schema(reference_path)?;

    let current = ag_dsl::compile(&current_source).map_err(|diags| {
        format!(
            "errors in current schema: {}",
            diags
                .iter()
                .filter(|d| d.is_error())
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    let reference = ag_dsl::compile(&ref_source).map_err(|diags| {
        format!(
            "errors in reference schema: {}",
            diags
                .iter()
                .filter(|d| d.is_error())
                .map(|d| d.message.clone())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

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

/// Compares two schemas and returns a list of readable changes.
///
/// Detects: added/removed models, added/removed/changed fields.
/// Breaking changes (removing a model, removing a field, changing a type) are
/// marked with `[BREAKING]`. Non-breaking changes are marked with `[additive]`.
pub(crate) fn diff_schemas(old: &ag_dsl::ast::Schema, new: &ag_dsl::ast::Schema) -> Vec<String> {
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
            changes.push(format!("[BREAKING]  model '{name}' removed"));
        }
    }

    // Added models (non-breaking)
    for name in new_models.keys() {
        if !old_models.contains_key(name) {
            changes.push(format!("[additive]  model '{name}' added"));
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
                    changes.push(format!("[BREAKING]  '{name}.{fname}' removed"));
                }
            }
            for fname in new_fields.keys() {
                if !old_fields.contains_key(fname) {
                    changes.push(format!("[additive]  '{name}.{fname}' added"));
                }
            }
            for (fname, old_field) in &old_fields {
                if let Some(new_field) = new_fields.get(fname) {
                    if old_field.ty.value != new_field.ty.value {
                        changes.push(format!(
                            "[BREAKING]  '{name}.{fname}' type changed: {:?} -> {:?}",
                            old_field.ty.value, new_field.ty.value
                        ));
                    }
                    if old_field.optional && !new_field.optional {
                        changes.push(format!(
                            "[BREAKING]  '{name}.{fname}' changed from nullable to NOT NULL"
                        ));
                    }
                    if !old_field.optional && new_field.optional {
                        changes.push(format!(
                            "[additive]  '{name}.{fname}' changed from NOT NULL to nullable"
                        ));
                    }
                }
            }
        }
    }

    changes
}

pub(crate) fn read_schema(path: &Path) -> Result<String, String> {
    if !path.exists() {
        return Err(format!("schema file not found: '{}'", path.display()));
    }
    fs::read_to_string(path).map_err(|e| format!("failed to read '{}': {e}", path.display()))
}
