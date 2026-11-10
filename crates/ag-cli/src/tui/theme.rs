//! Visual styles of the interactive interface.
//!
//! Two themes exist: color and monochrome. The monochrome theme is selected
//! when `NO_COLOR` is set. No information is conveyed by color alone: the
//! selected row also carries a marker and errors also carry a text prefix.

use ratatui::style::{Color, Modifier, Style};

/// Styles used by every screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Theme {
    pub(crate) title: Style,
    pub(crate) label: Style,
    pub(crate) selected: Style,
    pub(crate) error: Style,
    pub(crate) hint: Style,
    pub(crate) border: Style,
}

impl Theme {
    /// Theme for terminals where color is allowed.
    pub(crate) fn color() -> Theme {
        Theme {
            title: Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            label: Style::new().add_modifier(Modifier::BOLD),
            selected: Style::new().fg(Color::Black).bg(Color::Cyan),
            error: Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
            hint: Style::new().fg(Color::DarkGray),
            border: Style::new().fg(Color::DarkGray),
        }
    }

    /// Theme without any color, for `NO_COLOR`.
    pub(crate) fn monochrome() -> Theme {
        Theme {
            title: Style::new().add_modifier(Modifier::BOLD),
            label: Style::new().add_modifier(Modifier::BOLD),
            selected: Style::new().add_modifier(Modifier::REVERSED),
            error: Style::new().add_modifier(Modifier::BOLD),
            hint: Style::new(),
            border: Style::new(),
        }
    }

    /// Picks the theme for the session.
    pub(crate) fn for_session(no_color: bool) -> Theme {
        if no_color {
            Theme::monochrome()
        } else {
            Theme::color()
        }
    }
}
