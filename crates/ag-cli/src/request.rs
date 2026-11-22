//! Typed requests shared by the flag parser and the interactive interface.
//!
//! A command is described by one request value. The flag path and the TUI both
//! build the same value and hand it to the same executor, so the two cannot
//! drift apart (RFC-0018, principle 2).

// Parts of this module are used only by the interactive interface.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

// `Template` is defined by the catalog module, which is the single source of
// truth shared by the clap parser, the interactive selector and the scaffolder.
pub(crate) use crate::catalog::Template;

/// Metadata of one module that can be enabled in a generated `ag.toml`.
///
/// The names are the Anti-Gravital crates the layered architecture anticipates
/// (RFC-0019 section 4.1 and the crate roster). A module is recorded in the
/// manifest only; wiring its dependency into the generated crates is a later
/// stage and is deliberately not faked here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ModuleEntry {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

/// Declared module names, the single source of truth for `ag new --with`.
///
/// The order is stable: it is used by clap for help and error messages and by
/// the tests that check parity between the two lists.
pub(crate) const MODULES: &[ModuleEntry] = &[
    ModuleEntry {
        name: "data",
        description: "PostgreSQL persistence through ag-data",
    },
    ModuleEntry {
        name: "auth",
        description: "Authentication and sessions through ag-auth",
    },
    ModuleEntry {
        name: "cache",
        description: "Caching through ag-cache",
    },
    ModuleEntry {
        name: "mail",
        description: "Transactional mail through ag-mail",
    },
    ModuleEntry {
        name: "workers",
        description: "Background jobs through ag-workers",
    },
    ModuleEntry {
        name: "storage",
        description: "Object storage through ag-storage",
    },
    ModuleEntry {
        name: "realtime",
        description: "WebSocket transport through ag-realtime",
    },
    ModuleEntry {
        name: "observe",
        description: "Tracing and metrics through ag-observe",
    },
];

/// Values accepted by the clap `--with` parser, in declaration order.
pub(crate) const MODULE_NAMES: [&str; MODULES.len()] = [
    "data", "auth", "cache", "mail", "workers", "storage", "realtime", "observe",
];

/// A selected module. Holds a canonical name from [`MODULES`], so a value
/// cannot describe a module that was not declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Module {
    name: &'static str,
}

impl Module {
    /// Parses a `--with` value. Unknown names return `None`.
    pub(crate) fn parse(value: &str) -> Option<Module> {
        MODULES
            .iter()
            .find(|entry| entry.name == value)
            .map(|entry| Module { name: entry.name })
    }

    /// Canonical name used as the key in `ag.toml \[modules\]`.
    pub(crate) fn as_str(self) -> &'static str {
        self.name
    }
}

/// Everything `ag new` needs to create a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewProjectRequest {
    /// Project name, also used as the directory name.
    pub(crate) name: String,
    /// Starting template.
    pub(crate) template: Template,
    /// Modules to enable in the generated `ag.toml`. Empty leaves the default
    /// manifest byte-for-byte unchanged (RFC-0019 stage A).
    pub(crate) modules: Vec<Module>,
}

impl NewProjectRequest {
    /// The non-interactive command line that produces this exact request.
    ///
    /// Printed after an interactive flow so the action can be repeated and
    /// scripted (RFC-0018, principle 3).
    pub(crate) fn equivalent_command(&self) -> String {
        let mut command = format!("ag new {} --template {}", self.name, self.template);
        for module in &self.modules {
            command.push_str(&format!(" --with {}", module.as_str()));
        }
        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_names_round_trip() {
        for &template in Template::ALL {
            assert_eq!(Template::parse(template.as_str()), Some(template));
        }
        assert_eq!(Template::parse("unknown"), None);
    }

    #[test]
    fn equivalent_command_names_every_input() {
        let request = NewProjectRequest {
            name: "my-api".into(),
            template: Template::parse("fullstack").expect("cataloged"),
            modules: Vec::new(),
        };
        assert_eq!(
            request.equivalent_command(),
            "ag new my-api --template fullstack"
        );
    }

    #[test]
    fn module_names_match_the_registry() {
        assert_eq!(MODULE_NAMES.len(), MODULES.len());
        for (name, entry) in MODULE_NAMES.iter().zip(MODULES.iter()) {
            assert_eq!(name, &entry.name);
        }
    }

    #[test]
    fn every_declared_module_round_trips_through_parse() {
        for name in MODULE_NAMES {
            let module = Module::parse(name).unwrap_or_else(|| panic!("module '{name}' parses"));
            assert_eq!(module.as_str(), name);
        }
        assert_eq!(Module::parse("unknown"), None);
        assert_eq!(Module::parse(""), None);
    }

    #[test]
    fn equivalent_command_names_enabled_modules() {
        let request = NewProjectRequest {
            name: "my-api".into(),
            template: Template::parse("layered").expect("cataloged"),
            modules: vec![
                Module::parse("auth").expect("declared"),
                Module::parse("cache").expect("declared"),
            ],
        };
        assert_eq!(
            request.equivalent_command(),
            "ag new my-api --template layered --with auth --with cache"
        );
    }
}
