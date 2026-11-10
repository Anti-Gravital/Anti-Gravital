//! Typed requests shared by the flag parser and the interactive interface.
//!
//! A command is described by one request value. The flag path and the TUI both
//! build the same value and hand it to the same executor, so the two cannot
//! drift apart (RFC-0018, principle 2).

// Parts of this module are used only by the interactive interface.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use std::fmt;

/// Project templates offered by `ag new`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Template {
    /// Minimal REST API.
    Rest,
    /// WebSocket server.
    Realtime,
    /// REST API with PostgreSQL persistence and migrations.
    Fullstack,
}

impl Template {
    /// Every template, in the order shown to the user.
    pub(crate) const ALL: [Template; 3] = [Template::Rest, Template::Realtime, Template::Fullstack];

    /// Name accepted by `--template`.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Template::Rest => "rest",
            Template::Realtime => "realtime",
            Template::Fullstack => "fullstack",
        }
    }

    /// Parses a `--template` value.
    pub(crate) fn parse(value: &str) -> Option<Template> {
        Template::ALL.into_iter().find(|t| t.as_str() == value)
    }

    /// One-line description shown next to the template name.
    pub(crate) fn description(self) -> &'static str {
        match self {
            Template::Rest => "Minimal REST API on ag-core (Shield + axum) with one example route.",
            Template::Realtime => "WebSocket server on ag-core with a /ws route.",
            Template::Fullstack => {
                "REST API with PostgreSQL persistence through ag-data and SQL migrations."
            }
        }
    }

    /// Files the scaffolder writes for this template, relative to the project
    /// directory.
    pub(crate) fn files(self) -> &'static [&'static str] {
        match self {
            Template::Rest | Template::Realtime => &["Cargo.toml", "src/main.rs", "config.toml"],
            Template::Fullstack => &[
                "Cargo.toml",
                "src/main.rs",
                "config.toml",
                "migrations/0001_init.sql",
            ],
        }
    }
}

impl fmt::Display for Template {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Everything `ag new` needs to create a project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NewProjectRequest {
    /// Project name, also used as the directory name.
    pub(crate) name: String,
    /// Starting template.
    pub(crate) template: Template,
}

impl NewProjectRequest {
    /// The non-interactive command line that produces this exact request.
    ///
    /// Printed after an interactive flow so the action can be repeated and
    /// scripted (RFC-0018, principle 3).
    pub(crate) fn equivalent_command(&self) -> String {
        format!("ag new {} --template {}", self.name, self.template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_names_round_trip() {
        for template in Template::ALL {
            assert_eq!(Template::parse(template.as_str()), Some(template));
        }
        assert_eq!(Template::parse("unknown"), None);
    }

    #[test]
    fn equivalent_command_names_every_input() {
        let request = NewProjectRequest {
            name: "my-api".into(),
            template: Template::Fullstack,
        };
        assert_eq!(
            request.equivalent_command(),
            "ag new my-api --template fullstack"
        );
    }
}
