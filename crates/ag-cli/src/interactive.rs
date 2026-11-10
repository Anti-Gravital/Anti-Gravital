//! Decides whether a command may interact with the user.
//!
//! The rule (RFC-0018, principle 4): `ag` is never interactive by surprise.
//! Prompts and the full-screen interface are used only when every condition
//! holds; in any other case commands behave as plain, scriptable tools.

// Parts of this module are used only by the interactive interface.
#![cfg_attr(not(feature = "tui"), allow(dead_code))]

use std::io::IsTerminal;

/// Minimum terminal size for the full-screen interface. Below it the flow
/// degrades to sequential line prompts.
pub(crate) const MIN_COLUMNS: u16 = 80;
/// See [`MIN_COLUMNS`].
pub(crate) const MIN_ROWS: u16 = 24;

/// Facts about the session that decide interactivity. Gathered once from the
/// real environment in production and built by hand in tests.
#[derive(Debug, Clone, Default)]
pub(crate) struct Session {
    /// `--no-interactive` was passed.
    pub(crate) no_interactive_flag: bool,
    /// Standard input is a terminal.
    pub(crate) stdin_is_terminal: bool,
    /// Standard output is a terminal.
    pub(crate) stdout_is_terminal: bool,
    /// The `CI` environment variable is set to a non-empty value.
    pub(crate) ci: bool,
    /// The `AG_NO_TUI` environment variable is set to a non-empty value.
    pub(crate) ag_no_tui: bool,
    /// The `TERM` environment variable is `dumb`.
    pub(crate) term_is_dumb: bool,
    /// The `NO_COLOR` environment variable is set to a non-empty value.
    pub(crate) no_color: bool,
}

impl Session {
    /// Reads the session facts from the process environment.
    pub(crate) fn from_environment(no_interactive_flag: bool) -> Session {
        let set = |name: &str| std::env::var_os(name).is_some_and(|value| !value.is_empty());
        Session {
            no_interactive_flag,
            stdin_is_terminal: std::io::stdin().is_terminal(),
            stdout_is_terminal: std::io::stdout().is_terminal(),
            ci: set("CI"),
            ag_no_tui: set("AG_NO_TUI"),
            term_is_dumb: std::env::var_os("TERM").is_some_and(|value| value == "dumb"),
            no_color: set("NO_COLOR"),
        }
    }

    /// True when the command may ask the user anything at all.
    pub(crate) fn is_interactive(&self) -> bool {
        !self.no_interactive_flag
            && self.stdin_is_terminal
            && self.stdout_is_terminal
            && !self.ci
            && !self.ag_no_tui
            && !self.term_is_dumb
    }
}

/// True when a terminal of this size can host the full-screen interface.
pub(crate) fn fits_full_screen(columns: u16, rows: u16) -> bool {
    columns >= MIN_COLUMNS && rows >= MIN_ROWS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terminal_session() -> Session {
        Session {
            stdin_is_terminal: true,
            stdout_is_terminal: true,
            ..Session::default()
        }
    }

    #[test]
    fn a_plain_terminal_session_is_interactive() {
        assert!(terminal_session().is_interactive());
    }

    #[test]
    fn every_opt_out_disables_interaction() {
        let cases: [fn(&mut Session); 6] = [
            |s| s.no_interactive_flag = true,
            |s| s.stdin_is_terminal = false,
            |s| s.stdout_is_terminal = false,
            |s| s.ci = true,
            |s| s.ag_no_tui = true,
            |s| s.term_is_dumb = true,
        ];
        for apply in cases {
            let mut session = terminal_session();
            apply(&mut session);
            assert!(!session.is_interactive(), "{session:?}");
        }
    }

    #[test]
    fn no_color_does_not_disable_interaction() {
        let session = Session {
            no_color: true,
            ..terminal_session()
        };
        assert!(session.is_interactive());
    }

    #[test]
    fn full_screen_needs_eighty_by_twenty_four() {
        assert!(fits_full_screen(80, 24));
        assert!(fits_full_screen(120, 40));
        assert!(!fits_full_screen(79, 24));
        assert!(!fits_full_screen(80, 23));
    }
}
