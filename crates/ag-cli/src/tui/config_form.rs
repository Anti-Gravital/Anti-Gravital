//! Project configuration form: editable fields, inline validation, review.
//!
//! Follows RFC-0018 section 4.7: [`Form`] is plain data, [`Form::update`] is a
//! pure transition and [`view`] only reads state. No I/O happens here. The only
//! way out with an effect is [`Effect::Submit`], carrying the same
//! [`ConfigRequest`] the `ag config set` flags build.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use super::text;
use super::theme::Theme;
use crate::config::{validate_change, ConfigChange, ConfigKey, ConfigRequest};

/// Number of fields rendered around the selection (the list is longer than the
/// minimum terminal height; the window follows the selection).
const WINDOW: usize = 6;
/// Longest editable value.
const MAX_VALUE_LENGTH: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Fields,
    Review,
}

impl Step {
    fn number(self) -> usize {
        match self {
            Step::Fields => 1,
            Step::Review => 2,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Step::Fields => text::STEP_FIELDS,
            Step::Review => text::STEP_REVIEW,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Msg {
    Up,
    Down,
    Edit,
    Review,
    Char(char),
    Backspace,
    Enter,
    Back,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Submit(ConfigRequest),
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Field {
    key: ConfigKey,
    original: String,
    current: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Form {
    step: Step,
    fields: Vec<Field>,
    selected: usize,
    editing: bool,
    buffer: String,
    error: Option<String>,
}

impl Form {
    /// Starts the form with the current values read by the executor layer.
    pub(crate) fn new(values: Vec<(ConfigKey, String)>) -> Form {
        Form {
            step: Step::Fields,
            fields: values
                .into_iter()
                .map(|(key, value)| Field {
                    key,
                    original: value.clone(),
                    current: value,
                })
                .collect(),
            selected: 0,
            editing: false,
            buffer: String::new(),
            error: None,
        }
    }

    pub(crate) fn step(&self) -> Step {
        self.step
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.editing
    }

    /// The edits made so far, in form order.
    pub(crate) fn changes(&self) -> Vec<ConfigChange> {
        self.fields
            .iter()
            .filter(|field| field.current != field.original)
            .map(|field| ConfigChange {
                key: field.key,
                value: field.current.clone(),
            })
            .collect()
    }

    fn request(&self) -> ConfigRequest {
        ConfigRequest::Set {
            changes: self.changes(),
        }
    }

    fn revalidate(&mut self) {
        self.error = self
            .fields
            .get(self.selected)
            .and_then(|field| validate_change(field.key, &self.buffer).err());
    }

    pub(crate) fn update(&mut self, msg: Msg) -> Option<Effect> {
        match self.step {
            Step::Fields if self.editing => self.update_editing(msg),
            Step::Fields => self.update_fields(msg),
            Step::Review => self.update_review(msg),
        }
    }

    fn update_editing(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Char(c) => {
                if !c.is_control() && self.buffer.chars().count() < MAX_VALUE_LENGTH {
                    self.buffer.push(c);
                    self.revalidate();
                }
            }
            Msg::Backspace => {
                self.buffer.pop();
                self.revalidate();
            }
            Msg::Enter => {
                self.revalidate();
                if self.error.is_none() {
                    if let Some(field) = self.fields.get_mut(self.selected) {
                        field.current = self.buffer.clone();
                    }
                    self.editing = false;
                    self.buffer.clear();
                }
            }
            Msg::Back => {
                self.editing = false;
                self.buffer.clear();
                self.error = None;
            }
            Msg::Abort => return Some(Effect::Quit),
            Msg::Up | Msg::Down | Msg::Edit | Msg::Review => {}
        }
        None
    }

    fn update_fields(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Up => self.selected = self.selected.saturating_sub(1),
            Msg::Down => {
                if !self.fields.is_empty() {
                    self.selected = (self.selected + 1).min(self.fields.len() - 1);
                }
            }
            Msg::Edit => {
                if let Some(field) = self.fields.get(self.selected) {
                    self.editing = true;
                    self.buffer = field.current.clone();
                    self.error = None;
                }
            }
            Msg::Review => {
                self.step = Step::Review;
                self.error = None;
            }
            Msg::Back | Msg::Abort => return Some(Effect::Quit),
            Msg::Char(_) | Msg::Backspace | Msg::Enter => {}
        }
        None
    }

    fn update_review(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Enter => {
                if self.changes().is_empty() {
                    return Some(Effect::Quit);
                }
                Some(Effect::Submit(self.request()))
            }
            Msg::Back => {
                self.step = Step::Fields;
                None
            }
            Msg::Abort => Some(Effect::Quit),
            Msg::Up | Msg::Down | Msg::Edit | Msg::Review | Msg::Char(_) | Msg::Backspace => None,
        }
    }
}

/// Translates a key press into an action for the current mode.
pub(crate) fn msg_for(step: Step, editing: bool, key: KeyEvent) -> Option<Msg> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') => Some(Msg::Abort),
            _ => None,
        };
    }
    if editing {
        return match key.code {
            KeyCode::Enter => Some(Msg::Enter),
            KeyCode::Esc => Some(Msg::Back),
            KeyCode::Backspace => Some(Msg::Backspace),
            KeyCode::Char(c) => Some(Msg::Char(c)),
            _ => None,
        };
    }
    match (step, key.code) {
        (Step::Fields, KeyCode::Up | KeyCode::Char('k')) => Some(Msg::Up),
        (Step::Fields, KeyCode::Down | KeyCode::Char('j')) => Some(Msg::Down),
        (Step::Fields, KeyCode::Enter) => Some(Msg::Edit),
        (Step::Fields, KeyCode::Char('r')) => Some(Msg::Review),
        (Step::Fields, KeyCode::Esc | KeyCode::Char('q')) => Some(Msg::Back),
        (Step::Review, KeyCode::Enter) => Some(Msg::Enter),
        (Step::Review, KeyCode::Esc | KeyCode::Char('b')) => Some(Msg::Back),
        (Step::Review, KeyCode::Char('q')) => Some(Msg::Abort),
        _ => None,
    }
}

/// Draws the form. Reads the state and nothing else.
pub(crate) fn view(frame: &mut Frame, form: &Form, theme: &Theme) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    let heading = Line::from(vec![
        Span::styled(text::PRODUCT, theme.title),
        Span::raw("  |  "),
        Span::raw(text::CONFIGURE_PROJECT),
        Span::raw("  |  "),
        Span::styled(
            format!("Step {} of 2: {}", form.step.number(), form.step.title()),
            theme.label,
        ),
    ]);
    frame.render_widget(
        Paragraph::new(heading).block(Block::bordered().border_style(theme.border)),
        header,
    );

    let lines = match form.step {
        Step::Fields => fields_lines(form, theme),
        Step::Review => review_lines(form, theme),
    };
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(theme.border)),
        body,
    );

    let keys = match (form.step, form.editing) {
        (Step::Fields, true) => text::KEYS_CONFIG_EDIT,
        (Step::Fields, false) => text::KEYS_CONFIG_FIELDS,
        (Step::Review, _) => text::KEYS_CONFIG_REVIEW,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(keys, theme.hint))
            .block(Block::bordered().border_style(theme.border)),
        footer,
    );
}

fn fields_lines<'a>(form: &'a Form, theme: &Theme) -> Vec<Line<'a>> {
    let first = form.selected.saturating_sub(WINDOW / 2);
    let last = (first + WINDOW).min(form.fields.len());
    let mut lines = Vec::new();
    let mut previous_section = "";
    for (index, field) in form.fields[first..last].iter().enumerate() {
        let absolute = first + index;
        if field.key.section() != previous_section {
            lines.push(Line::from(Span::styled(
                format!("[{}]", field.key.section()),
                theme.label,
            )));
            previous_section = field.key.section();
        }
        let selected = absolute == form.selected;
        let marker = if selected { "> " } else { "  " };
        let shown = if selected && form.editing {
            format!("{}_", form.buffer)
        } else {
            field.current.clone()
        };
        let style = if selected {
            theme.selected
        } else {
            ratatui::style::Style::new()
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{marker}{:<28}", field.key.as_str()), style),
            Span::styled(shown, style),
        ]));
        if selected && !form.editing {
            lines.push(Line::from(Span::styled(
                format!("    {}", field.key.description()),
                theme.hint,
            )));
        }
    }
    if let Some(error) = &form.error {
        lines.push(Line::from(Span::styled(
            format!("{}{error}", text::ERROR_PREFIX),
            theme.error,
        )));
    }
    lines
}

fn review_lines<'a>(form: &'a Form, theme: &Theme) -> Vec<Line<'a>> {
    let changes = form.changes();
    let mut lines = vec![
        Line::from(Span::styled(text::CONFIG_REVIEW_LABEL, theme.label)),
        Line::raw(""),
    ];
    for change in &changes {
        let original = form
            .fields
            .iter()
            .find(|field| field.key == change.key)
            .map(|field| field.original.as_str())
            .unwrap_or("");
        lines.push(Line::raw(format!(
            "{}: {} -> {}",
            change.key.as_str(),
            original,
            change.value
        )));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(text::REVIEW_COMMAND, theme.label)));
    for change in &changes {
        lines.push(Line::raw(format!("  {}", change.equivalent_command())));
    }
    lines.push(Line::raw(""));
    lines.push(Line::from(Span::styled(
        text::REVIEW_NOTHING_WRITTEN,
        theme.hint,
    )));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::Terminal;

    fn sample_values() -> Vec<(ConfigKey, String)> {
        vec![
            (ConfigKey::Bind, "0.0.0.0:8080".into()),
            (ConfigKey::RateLimitEnabled, "false".into()),
            (ConfigKey::RateLimitPerIpRps, "100".into()),
        ]
    }

    fn values_with_all_sections() -> Vec<(ConfigKey, String)> {
        ConfigKey::ALL
            .into_iter()
            .map(|key| (key, "0".to_owned()))
            .collect()
    }

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render(form: &Form, theme: &Theme) -> String {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("terminal");
        terminal
            .draw(|frame| view(frame, form, theme))
            .expect("draw");
        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn editing_a_field_then_reviewing_submits_the_request() {
        let mut form = Form::new(sample_values());
        assert_eq!(form.update(Msg::Down), None);
        assert_eq!(form.update(Msg::Edit), None);
        clear_buffer(&mut form);
        for c in "true".chars() {
            assert_eq!(form.update(Msg::Char(c)), None);
        }
        assert_eq!(form.update(Msg::Enter), None);
        assert_eq!(form.update(Msg::Review), None);
        assert_eq!(
            form.update(Msg::Enter),
            Some(Effect::Submit(ConfigRequest::Set {
                changes: vec![ConfigChange {
                    key: ConfigKey::RateLimitEnabled,
                    value: "true".into(),
                }],
            }))
        );
    }

    fn clear_buffer(form: &mut Form) {
        for _ in 0..form.buffer.len() {
            form.update(Msg::Backspace);
        }
    }

    #[test]
    fn inline_validation_blocks_an_invalid_value_and_can_be_corrected() {
        let mut form = Form::new(sample_values());
        form.update(Msg::Edit);
        clear_buffer(&mut form);
        form.update(Msg::Char('x'));
        assert!(form.error.is_some());
        assert_eq!(form.update(Msg::Enter), None);
        assert!(form.editing, "invalid value must not be committed");
        form.update(Msg::Backspace);
        for c in "1.2.3.4:123".chars() {
            form.update(Msg::Char(c));
        }
        assert_eq!(form.error, None);
        assert_eq!(form.update(Msg::Enter), None);
        assert!(!form.editing);
        assert_eq!(form.changes()[0].value, "1.2.3.4:123");
    }

    #[test]
    fn back_from_a_field_and_abort_from_anywhere_quits() {
        let mut form = Form::new(sample_values());
        assert_eq!(form.update(Msg::Back), Some(Effect::Quit));
        let mut form = Form::new(sample_values());
        form.update(Msg::Review);
        assert_eq!(form.update(Msg::Abort), Some(Effect::Quit));
        let mut form = Form::new(sample_values());
        form.update(Msg::Edit);
        form.update(Msg::Char('x'));
        assert_eq!(form.update(Msg::Abort), Some(Effect::Quit));
    }

    #[test]
    fn review_back_returns_to_fields_and_empty_changes_do_not_write() {
        let mut form = Form::new(sample_values());
        form.update(Msg::Review);
        assert_eq!(form.update(Msg::Back), None);
        assert_eq!(form.step(), Step::Fields);
        form.update(Msg::Review);
        assert_eq!(form.update(Msg::Enter), Some(Effect::Quit));
    }

    #[test]
    fn selection_stays_inside_the_list() {
        let mut form = Form::new(values_with_all_sections());
        for _ in 0..50 {
            form.update(Msg::Up);
        }
        assert_eq!(form.selected, 0);
        for _ in 0..50 {
            form.update(Msg::Down);
        }
        assert_eq!(form.selected, ConfigKey::ALL.len() - 1);
        form.update(Msg::Down);
        assert_eq!(form.selected, ConfigKey::ALL.len() - 1);
    }

    #[test]
    fn key_mapping_is_mode_aware() {
        assert_eq!(
            msg_for(Step::Fields, false, press(KeyCode::Char('q'))),
            Some(Msg::Back)
        );
        assert_eq!(
            msg_for(Step::Fields, true, press(KeyCode::Char('q'))),
            Some(Msg::Char('q'))
        );
        assert_eq!(
            msg_for(Step::Fields, false, press(KeyCode::Char('r'))),
            Some(Msg::Review)
        );
        assert_eq!(
            msg_for(Step::Review, false, press(KeyCode::Char('b'))),
            Some(Msg::Back)
        );
        let mut release = press(KeyCode::Enter);
        release.kind = KeyEventKind::Release;
        assert_eq!(msg_for(Step::Fields, false, release), None);
    }

    #[test]
    fn rendering_shows_sections_selection_and_equivalent_commands() {
        let form = Form::new(values_with_all_sections());
        let screen = render(&form, &Theme::color());
        assert!(screen.contains("Step 1 of 2: Fields"), "{screen}");
        assert!(screen.contains("[Bind address]"), "{screen}");
        assert!(screen.contains("> bind"), "{screen}");
        assert!(screen.contains(text::KEYS_CONFIG_FIELDS), "{screen}");

        let mut form = Form::new(sample_values());
        form.update(Msg::Down);
        form.update(Msg::Edit);
        clear_buffer(&mut form);
        for c in "true".chars() {
            form.update(Msg::Char(c));
        }
        form.update(Msg::Enter);
        form.update(Msg::Review);
        let screen = render(&form, &Theme::color());
        assert!(screen.contains("Step 2 of 2: Review"), "{screen}");
        assert!(
            screen.contains("rate_limit.enabled: false -> true"),
            "{screen}"
        );
        assert!(
            screen.contains("ag config set rate_limit.enabled true"),
            "{screen}"
        );
    }

    #[test]
    fn error_is_rendered_with_a_text_prefix_and_monochrome_has_no_color() {
        let mut form = Form::new(sample_values());
        form.update(Msg::Edit);
        form.update(Msg::Char('x'));
        let color_screen = render(&form, &Theme::color());
        assert!(color_screen.contains(text::ERROR_PREFIX), "{color_screen}");
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("terminal");
        terminal
            .draw(|frame| view(frame, &form, &Theme::monochrome()))
            .expect("draw");
        for cell in terminal.backend().buffer().content() {
            assert_eq!(cell.fg, Color::Reset);
            assert_eq!(cell.bg, Color::Reset);
        }
    }

    #[test]
    fn rendered_glyphs_are_ascii_or_box_drawing_only() {
        let form = Form::new(values_with_all_sections());
        let screen = render(&form, &Theme::color());
        for c in screen.chars() {
            let box_drawing = ('\u{2500}'..='\u{257F}').contains(&c);
            assert!(c.is_ascii() || box_drawing, "unexpected glyph {c:?}");
        }
    }
}
