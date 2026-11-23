//! Interactive terminal interface of the `ag` binary (RFC-0018).
//!
//! The interface is a front-end. A screen never performs work: it returns the
//! same typed request the flag parser builds, and the caller executes it.
//! This module is compiled only with the `tui` Cargo feature.

mod config_form;
mod home;
mod project_context;
mod line;
mod new_wizard;
mod terminal;
pub(crate) mod text;
mod theme;

use std::io;
use std::path::Path;

use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::event::{self, Event};
use ratatui::Terminal;

use crate::config::{ConfigKey, ConfigRequest};
use crate::interactive::{fits_full_screen, Session};
use crate::request::{NewProjectRequest, Template};
use config_form::{msg_for as config_msg_for, view as config_view, Effect as ConfigEffect, Form};
use home::{
    msg_for as home_msg_for, view as home_view, Effect as HomeEffect, Model as HomeModel,
    Msg as HomeMsg,
};
use new_wizard::{msg_for, view, Effect, Wizard};
use project_context::detect_context;
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

/// Opens the configuration form. Returns the typed request the flags could
/// have built; the caller executes it. Returns `None` on cancel.
pub(crate) fn ask_config(
    values: &[(ConfigKey, String)],
    session: &Session,
) -> Result<Option<ConfigRequest>, String> {
    let full_screen = ratatui::crossterm::terminal::size()
        .map(|(columns, rows)| fits_full_screen(columns, rows))
        .unwrap_or(false);
    if !full_screen {
        return Err(
            "the terminal is too small for the configuration form (minimum 80x24)".to_owned(),
        );
    }
    run_config_form(Form::new(values.to_vec()), session.no_color)
        .map_err(|error| format!("interactive session failed: {error}"))
}

fn run_config_form(mut form: Form, no_color: bool) -> io::Result<Option<ConfigRequest>> {
    let theme = Theme::for_session(no_color);
    let _guard = Guard::enter(CrosstermOps)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    loop {
        terminal.draw(|frame| config_view(frame, &form, &theme))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let Some(msg) = config_msg_for(form.step(), form.is_editing(), key) else {
            continue;
        };
        match form.update(msg) {
            Some(ConfigEffect::Submit(request)) => return Ok(Some(request)),
            Some(ConfigEffect::Quit) => return Ok(None),
            None => {}
        }
    }
}

/// Opens the home screen of bare `ag`. Returns the command line chosen by the
/// user. The screen itself performs no work: the caller re-parses and runs it.
pub(crate) fn run_home(session: &Session) -> Option<String> {
    let full_screen = ratatui::crossterm::terminal::size()
        .map(|(columns, rows)| fits_full_screen(columns, rows))
        .unwrap_or(false);
    if !full_screen {
        return None;
    }
    let model = HomeModel::new(in_project());
    run_home_loop(model, session.no_color).ok().flatten()
}

fn in_project() -> bool {
    if !Path::new("config.toml").is_file() {
        return false;
    }
    std::fs::read_to_string("Cargo.toml").is_ok_and(|manifest| manifest.contains("ag-core"))
}

fn run_home_loop(mut model: HomeModel, no_color: bool) -> io::Result<Option<String>> {
    let theme = Theme::for_session(no_color);
    let _guard = Guard::enter(CrosstermOps)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    loop {
        terminal.draw(|frame| home_view(frame, &model, &theme))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        let Some(msg) = home_msg_for(key) else {
            continue;
        };
        match model.update(msg) {
            Some(HomeEffect::Launch(command)) => return Ok(Some(command)),
            Some(HomeEffect::Quit) => return Ok(None),
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_entries_are_launchers_with_no_side_effects() {
        let mut model = HomeModel::new(false);
        let effect = model.update(home_msg_for(press(KeyCode::Enter)).expect("msg"));
        assert_eq!(effect, Some(HomeEffect::Launch("ag new".into())));
    }

    #[test]
    fn config_form_produces_a_request_without_executing_it() {
        let mut form = Form::new(vec![(ConfigKey::Bind, "0.0.0.0:8080".into())]);
        // A form with no changes cannot submit; that is intentional.
        assert_eq!(form.update(config_form::Msg::Review), None);
        assert_eq!(
            form.update(config_form::Msg::Enter),
            Some(ConfigEffect::Quit)
        );
    }

    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }
}
