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
        };
        assert_eq!(
            request.equivalent_command(),
            "ag new my-api --template fullstack"
        );
    }
}
