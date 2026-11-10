//! Project creation wizard: name, template, review.
//!
//! The screen follows the pattern of RFC-0018 section 4.7. [`Wizard`] is plain
//! data, [`Wizard::update`] is a pure state transition and [`view`] only reads
//! the state. Neither performs I/O. The only way out with an effect is
//! [`Effect::Submit`], which carries the same [`NewProjectRequest`] the flag
//! parser builds.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use super::text;
use super::theme::Theme;
use crate::commands::project::validate_project_name;
use crate::request::{NewProjectRequest, Template};

/// Longest project name the input field accepts.
const MAX_NAME_LENGTH: usize = 64;

/// The three steps of the wizard, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Name,
    Template,
    Review,
}

impl Step {
    fn number(self) -> usize {
        match self {
            Step::Name => 1,
            Step::Template => 2,
            Step::Review => 3,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Step::Name => text::STEP_NAME,
            Step::Template => text::STEP_TEMPLATE,
            Step::Review => text::STEP_REVIEW,
        }
    }
}

/// A user action, already translated from a key press.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Msg {
    /// A printable character typed into the name field.
    Char(char),
    /// Delete the last character of the name.
    Backspace,
    /// Confirm the current step.
    Enter,
    /// Return to the previous step, or cancel on the first one.
    Back,
    /// Move the template selection up.
    Up,
    /// Move the template selection down.
    Down,
    /// Leave the wizard without creating anything.
    Abort,
}

/// What the event loop must do after a transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    /// Create the project described by the request.
    Submit(NewProjectRequest),
    /// Leave without writing anything.
    Quit,
}

/// Wizard state.
#[derive(Debug, Clone)]
pub(crate) struct Wizard {
    step: Step,
    name: String,
    error: Option<String>,
    selected: usize,
    /// Reports whether a path with this name already exists. Injected so the
    /// state machine stays free of file-system access.
    name_taken: fn(&str) -> bool,
}

impl Wizard {
    /// Starts the wizard. Inputs already given on the command line are
    /// prefilled; a valid prefilled name skips straight to the template step.
    pub(crate) fn new(
        name: Option<String>,
        template: Option<Template>,
        name_taken: fn(&str) -> bool,
    ) -> Wizard {
        let selected = template
            .and_then(|wanted| Template::ALL.iter().position(|t| *t == wanted))
            .unwrap_or(0);
        let mut wizard = Wizard {
            step: Step::Name,
            name: name.unwrap_or_default(),
            error: None,
            selected,
            name_taken,
        };
        if !wizard.name.is_empty() {
            wizard.error = wizard.name_problem();
            if wizard.error.is_none() {
                wizard.step = Step::Template;
            }
        }
        wizard
    }

    /// Current step.
    pub(crate) fn step(&self) -> Step {
        self.step
    }

    /// The request described by the current state.
    pub(crate) fn request(&self) -> NewProjectRequest {
        NewProjectRequest {
            name: self.name.clone(),
            template: Template::ALL[self.selected],
        }
    }

    /// Why the current name cannot be used, if it cannot.
    fn name_problem(&self) -> Option<String> {
        if let Err(problem) = validate_project_name(&self.name) {
            return Some(problem);
        }
        if (self.name_taken)(&self.name) {
            return Some(text::NAME_TAKEN.to_owned());
        }
        None
    }

    /// Applies one user action. Returns an effect only when the wizard ends.
    pub(crate) fn update(&mut self, msg: Msg) -> Option<Effect> {
        if msg == Msg::Abort {
            return Some(Effect::Quit);
        }
        match self.step {
            Step::Name => self.update_name(msg),
            Step::Template => self.update_template(msg),
            Step::Review => self.update_review(msg),
        }
    }

    fn update_name(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Char(c) => {
                if !c.is_control() && self.name.chars().count() < MAX_NAME_LENGTH {
                    self.name.push(c);
                    self.error = self.name_problem();
                }
            }
            Msg::Backspace => {
                self.name.pop();
                self.error = if self.name.is_empty() {
                    None
                } else {
                    self.name_problem()
                };
            }
            Msg::Enter => {
                self.error = self.name_problem();
                if self.error.is_none() {
                    self.step = Step::Template;
                }
            }
            Msg::Back => return Some(Effect::Quit),
            Msg::Up | Msg::Down | Msg::Abort => {}
        }
        None
    }

    fn update_template(&mut self, msg: Msg) -> Option<Effect> {
        let last = Template::ALL.len() - 1;
        match msg {
            Msg::Up => self.selected = self.selected.saturating_sub(1),
            Msg::Down => self.selected = (self.selected + 1).min(last),
            Msg::Enter => self.step = Step::Review,
            Msg::Back => self.step = Step::Name,
            Msg::Char(_) | Msg::Backspace | Msg::Abort => {}
        }
        None
    }

    fn update_review(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Enter => {
                // The directory may have appeared while the user was reading.
                self.error = self.name_problem();
                if self.error.is_some() {
                    self.step = Step::Name;
                    return None;
                }
                Some(Effect::Submit(self.request()))
            }
            Msg::Back => {
                self.step = Step::Template;
                None
            }
            Msg::Char(_) | Msg::Backspace | Msg::Up | Msg::Down | Msg::Abort => None,
        }
    }
}

/// Translates a key press into an action for the given step. Key releases and
/// repeats reported by some terminals are ignored.
pub(crate) fn msg_for(step: Step, key: KeyEvent) -> Option<Msg> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') => Some(Msg::Abort),
            _ => None,
        };
    }
    match (step, key.code) {
        (_, KeyCode::Enter) => Some(Msg::Enter),
        (_, KeyCode::Esc) => Some(Msg::Back),
        (Step::Name, KeyCode::Backspace) => Some(Msg::Backspace),
        (Step::Name, KeyCode::Char(c)) => Some(Msg::Char(c)),
        (Step::Template, KeyCode::Up | KeyCode::Char('k')) => Some(Msg::Up),
        (Step::Template, KeyCode::Down | KeyCode::Char('j')) => Some(Msg::Down),
        (Step::Template | Step::Review, KeyCode::Char('q')) => Some(Msg::Abort),
        (Step::Template | Step::Review, KeyCode::Char('b')) => Some(Msg::Back),
        _ => None,
    }
}

/// Draws the wizard. Reads the state and nothing else.
pub(crate) fn view(frame: &mut Frame, wizard: &Wizard, theme: &Theme) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let step = wizard.step;
    let heading = Line::from(vec![
        Span::styled(text::PRODUCT, theme.title),
        Span::raw("  |  "),
        Span::raw(text::NEW_PROJECT),
        Span::raw("  |  "),
        Span::styled(
            format!("Step {} of 3: {}", step.number(), step.title()),
            theme.label,
        ),
    ]);
    frame.render_widget(
        Paragraph::new(heading).block(Block::bordered().border_style(theme.border)),
        header,
    );

    let lines = match step {
        Step::Name => name_lines(wizard, theme),
        Step::Template => template_lines(wizard, theme),
        Step::Review => review_lines(wizard, theme),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(theme.border)),
        body,
    );

    let keys = match step {
        Step::Name => text::KEYS_NAME,
        Step::Template => text::KEYS_TEMPLATE,
        Step::Review => text::KEYS_REVIEW,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(keys, theme.hint))
            .block(Block::bordered().border_style(theme.border)),
        footer,
    );
}

fn name_lines<'a>(wizard: &'a Wizard, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = vec![
        Line::from(Span::styled(text::NAME_LABEL, theme.label)),
        Line::raw(""),
        Line::from(vec![
            Span::raw("> "),
            Span::raw(wizard.name.as_str()),
            Span::raw("_"),
        ]),
        Line::raw(""),
    ];
    match &wizard.error {
        Some(problem) => lines.push(Line::from(Span::styled(
            format!("{}{problem}", text::ERROR_PREFIX),
            theme.error,
        ))),
        None => lines.push(Line::from(Span::styled(text::NAME_HINT, theme.hint))),
    }
    lines
}

fn template_lines<'a>(wizard: &'a Wizard, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = vec![
        Line::from(Span::styled(text::TEMPLATE_LABEL, theme.label)),
        Line::raw(""),
    ];
    for (index, template) in Template::ALL.iter().enumerate() {
        let chosen = index == wizard.selected;
        let marker = if chosen { "> " } else { "  " };
        let style = if chosen {
            theme.selected
        } else {
            ratatui::style::Style::new()
        };
        lines.push(Line::from(Span::styled(
            format!("{marker}{:<10}", template.as_str()),
            style,
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", template.description()),
            theme.hint,
        )));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(text::FILES_LABEL, theme.label)));
    for file in Template::ALL[wizard.selected].files() {
        lines.push(Line::raw(format!("  {file}")));
    }
    lines
}

fn review_lines<'a>(wizard: &'a Wizard, theme: &Theme) -> Vec<Line<'a>> {
    let request = wizard.request();
    let mut lines = vec![
        Line::from(Span::styled(text::REVIEW_LABEL, theme.label)),
        Line::raw(""),
        Line::raw(format!("{}: ./{}", text::REVIEW_DIRECTORY, request.name)),
        Line::raw(format!("{}: {}", text::REVIEW_TEMPLATE, request.template)),
        Line::raw(""),
        Line::from(Span::styled(text::REVIEW_FILES, theme.label)),
    ];
    for file in request.template.files() {
        lines.push(Line::raw(format!("  {}/{file}", request.name)));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(text::REVIEW_COMMAND, theme.label)));
    lines.push(Line::raw(format!("  {}", request.equivalent_command())));
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        text::REVIEW_NOTHING_WRITTEN,
        theme.hint,
    )));
    lines
}

#[cfg(test)]
mod tests;
