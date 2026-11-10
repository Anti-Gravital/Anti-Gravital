//! Interactive terminal interface of the `ag` binary (RFC-0018, stage A).
//!
//! The interface is a front-end. A screen never performs work: it returns the
//! same typed request the flag parser builds, and the caller executes it.
//! This module is compiled only with the `tui` Cargo feature.

mod line;
mod new_wizard;
mod terminal;
pub(crate) mod text;
mod theme;

use std::io;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::Terminal;

use crate::interactive::{fits_full_screen, Session};
use crate::request::{NewProjectRequest, Template};
use new_wizard::{msg_for, view, Effect, Wizard};
use terminal::{CrosstermOps, Guard};
use theme::Theme;

/// Asks the user for the inputs of `ag new` that were not given on the
/// command line. Returns `Ok(None)` when the user cancels.
///
/// Uses the full-screen wizard when the terminal is at least 80x24 and
/// sequential line prompts otherwise. The caller must already have checked
/// that the session is interactive.
pub(crate) fn ask_new_project(
    name: Option<String>,
    template: Option<Template>,
    session: &Session,
    name_taken: fn(&str) -> bool,
) -> Result<Option<NewProjectRequest>, String> {
    let full_screen = ratatui::crossterm::terminal::size()
        .map(|(columns, rows)| fits_full_screen(columns, rows))
        .unwrap_or(false);
    let outcome = if full_screen {
        run_wizard(Wizard::new(name, template, name_taken), session.no_color)
    } else {
        let stdin = io::stdin();
        line::prompt_new_project(
            &mut stdin.lock(),
            &mut io::stdout(),
            name,
            template,
            name_taken,
        )
    };
    outcome.map_err(|error| format!("interactive session failed: {error}"))
}

/// Event loop of the wizard: the only place that touches the terminal.
fn run_wizard(mut wizard: Wizard, no_color: bool) -> io::Result<Option<NewProjectRequest>> {
    let theme = Theme::for_session(no_color);
    let _guard = Guard::enter(CrosstermOps)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    loop {
        terminal.draw(|frame| view(frame, &wizard, &theme))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let Some(msg) = msg_for(wizard.step(), key) else {
            continue;
        };
        match wizard.update(msg) {
            Some(Effect::Submit(request)) => return Ok(Some(request)),
            Some(Effect::Quit) => return Ok(None),
            None => {}
        }
    }
}
