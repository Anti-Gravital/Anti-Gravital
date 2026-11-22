//! Data-driven scaffolding of projects from embedded templates.
//!
//! Template files are discovered and embedded by `build.rs` (see that file),
//! so a new `templates/<name>/` directory becomes scaffoldable without any
//! change to this module. `{{name}}` is the only substitution.

use std::path::Path;

use crate::commands::project::{apply_template, write_file};

include!(concat!(env!("OUT_DIR"), "/templates_generated.rs"));

/// File paths of a template, relative to the project directory.
///
/// Returns an empty slice for a template whose directory is not present in
/// this build; the catalog can list templates ahead of their implementation.
#[allow(dead_code)] // Used by the optional TUI selectors to list scaffolded files.
pub(crate) fn files_for(template: &str) -> &'static [&'static str] {
    GENERATED_FILE_LISTS
        .iter()
        .find(|(name, _)| *name == template)
        .map(|(_, files)| *files)
        .unwrap_or(&[])
}

/// Writes every file of `template` into `dir`, substituting the project name.
pub(crate) fn scaffold(name: &str, template: &str, dir: &Path) -> Result<(), String> {
    let Some((_, files)) = GENERATED_TEMPLATES
        .iter()
        .find(|(template_name, _)| *template_name == template)
    else {
        return Err(format!(
            "template '{template}' is cataloged but has no scaffold in this build"
        ));
    };
    for (relative, content) in *files {
        write_file(&dir.join(relative), &apply_template(content, name))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{Template, TEMPLATE_NAMES};

    #[test]
    fn the_rest_template_is_embedded_with_its_three_files() {
        let files = files_for("rest");
        assert!(files.contains(&"Cargo.toml"), "{files:?}");
        assert!(files.contains(&"src/main.rs"), "{files:?}");
        assert!(files.contains(&"config.toml"), "{files:?}");
    }

    #[test]
    fn template_toml_is_never_scaffolded() {
        for (_, files) in GENERATED_FILE_LISTS {
            assert!(!files.contains(&"template.toml"));
        }
    }

    #[test]
    fn a_missing_scaffold_reports_an_error_instead_of_an_empty_project() {
        let dir = std::path::Path::new("/nonexistent/ag-catalog-test");
        let error = scaffold("demo", "does-not-exist", dir).expect_err("must fail");
        assert!(error.contains("no scaffold"), "{error}");
    }

    #[test]
    fn every_cataloged_template_can_be_looked_up_without_panicking() {
        for name in TEMPLATE_NAMES {
            let _ = Template::parse(name).expect("catalog name parses");
            let _ = files_for(name);
        }
    }
}
