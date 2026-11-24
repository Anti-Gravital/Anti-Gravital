//! Home screen of bare `ag` (RFC-0018 stage C).
//!
//! A launcher only. [`Model`] holds plain data, [`Model::update`] returns a
//! command line as [`Effect::Launch`] and never performs work. The event loop
//! hands that string to the caller, which re-parses it through the real CLI so
//! the selected entry runs exactly the command the user could have typed.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::Frame;

use super::text;
use super::theme::Theme;

/// One menu entry: what it says and the command it launches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) command: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Model {
    context: &'static str,
    entries: Vec<Entry>,
    selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Msg {
    Up,
    Down,
    Enter,
    Back,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Launch(String),
    Quit,
}

impl Model {
    /// Builds the menu for the detected context.
    pub(crate) fn new(in_project: bool) -> Model {
        let entries = if in_project {
            let mut entries = vec![
                Entry {
                    label: "Run the development server".into(),
                    detail: "Hot reload while you edit (requires cargo-watch).".into(),
                    command: "ag dev".into(),
                },
                Entry {
                    label: "Generate artifacts from schema.ag".into(),
                    detail: "Rust, SQL, TypeScript and OpenAPI output.".into(),
                    command: "ag generate".into(),
                },
                Entry {
                    label: "Lint the DSL schema".into(),
                    detail: "Report best-practice warnings in schema.ag.".into(),
                    command: "ag schema lint".into(),
                },
                Entry {
                    label: "Configure this project".into(),
                    detail: "Open the config.toml form.".into(),
                    command: "ag config".into(),
                },
                Entry {
                    label: "List attached domains".into(),
                    detail: "Read-only status from the local attachment store.".into(),
                    command: "ag domains list".into(),
                },
                Entry {
                    label: "List background workers".into(),
                    detail: "Workers declared in schema.ag, no infrastructure needed.".into(),
                    command: "ag workers list".into(),
                },
            ];
            #[cfg(feature = "workers-runtime")]
            entries.push(Entry {
                label: "Check the workers backend".into(),
                detail: "Connectivity and configuration of the durable queue.".into(),
                command: "ag workers doctor".into(),
            });
            entries
        } else {
            vec![
                Entry {
                    label: "Create a new project".into(),
                    detail: "Guided creation with a name and a starting template.".into(),
                    command: "ag new".into(),
                },
                Entry {
                    label: "Help and documentation".into(),
                    detail: "Full command reference and usage.".into(),
                    command: "ag --help".into(),
                },
                Entry {
                    label: "Show the version".into(),
                    detail: "Print the installed ag version.".into(),
                    command: "ag --version".into(),
                },
            ]
        };
        Model {
            context: if in_project {
                text::HOME_IN_PROJECT
            } else {
                text::HOME_OUTSIDE_PROJECT
            },
            entries,
            selected: 0,
        }
    }

    pub(crate) fn update(&mut self, msg: Msg) -> Option<Effect> {
        match msg {
            Msg::Up => self.selected = self.selected.saturating_sub(1),
            Msg::Down => {
                if !self.entries.is_empty() {
                    self.selected = (self.selected + 1).min(self.entries.len() - 1);
                }
            }
            Msg::Enter => {
                return self
                    .entries
                    .get(self.selected)
                    .map(|entry| Effect::Launch(entry.command.clone()));
            }
            Msg::Back => return Some(Effect::Quit),
        }
        None
    }
}

/// Keyboard-only mapping. Key releases are ignored.
pub(crate) fn msg_for(key: KeyEvent) -> Option<Msg> {
    if key.kind != KeyEventKind::Press {
        return None;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') | KeyCode::Char('d') => Some(Msg::Back),
            _ => None,
        };
    }
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => Some(Msg::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Msg::Down),
        KeyCode::Enter => Some(Msg::Enter),
        KeyCode::Esc | KeyCode::Char('q') => Some(Msg::Back),
        _ => None,
    }
}

/// Draws the menu. Reads the model and nothing else.
pub(crate) fn view(frame: &mut Frame, model: &Model, theme: &Theme) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(5),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(text::PRODUCT, theme.title),
            Span::raw("  |  "),
            Span::raw(text::HOME_TITLE),
        ]))
        .block(Block::bordered().border_style(theme.border)),
        header,
    );

    let mut lines = vec![
        Line::from(Span::styled(model.context, theme.label)),
        Line::raw(""),
    ];
    for (index, entry) in model.entries.iter().enumerate() {
        let chosen = index == model.selected;
        let marker = if chosen { "> " } else { "  " };
        let style = if chosen {
            theme.selected
        } else {
            ratatui::style::Style::new()
        };
        lines.push(Line::from(Span::styled(
            format!("{marker}{}", entry.label),
            style,
        )));
        lines.push(Line::from(Span::styled(
            format!("    {}", entry.detail),
            theme.hint,
        )));
    }
    frame.render_widget(
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(theme.border)),
        body,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(text::KEYS_HOME, theme.hint))
            .block(Block::bordered().border_style(theme.border)),
        footer,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::Terminal;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn render(model: &Model, theme: &Theme) -> String {
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("terminal");
        terminal
            .draw(|frame| view(frame, model, theme))
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
    fn outside_project_offers_creation_help_and_version() {
        let model = Model::new(false);
        let commands: Vec<_> = model.entries.iter().map(|e| e.command.as_str()).collect();
        assert_eq!(commands, ["ag new", "ag --help", "ag --version"]);
    }

    #[test]
    fn inside_project_offers_the_documented_launcher_entries() {
        let model = Model::new(true);
        let commands: Vec<_> = model.entries.iter().map(|e| e.command.as_str()).collect();
        assert!(commands.contains(&"ag dev"));
        assert!(commands.contains(&"ag generate"));
        assert!(commands.contains(&"ag schema lint"));
        assert!(commands.contains(&"ag config"));
        assert!(commands.contains(&"ag domains list"));
        assert!(commands.contains(&"ag workers list"));
    }

    #[test]
    fn navigation_wraps_at_the_ends_of_the_list() {
        let mut model = Model::new(false);
        for _ in 0..10 {
            model.update(Msg::Up);
        }
        assert_eq!(model.selected, 0);
        for _ in 0..10 {
            model.update(Msg::Down);
        }
        assert_eq!(model.selected, model.entries.len() - 1);
    }

    #[test]
    fn enter_returns_a_command_without_performing_it() {
        let mut model = Model::new(false);
        assert_eq!(
            model.update(Msg::Enter),
            Some(Effect::Launch("ag new".into()))
        );
        model.update(Msg::Down);
        assert_eq!(
            model.update(Msg::Enter),
            Some(Effect::Launch("ag --help".into()))
        );
    }

    #[test]
    fn back_quits_from_the_menu() {
        let mut model = Model::new(true);
        assert_eq!(model.update(Msg::Back), Some(Effect::Quit));
    }

    #[test]
    fn key_mapping_is_keyboard_only() {
        assert_eq!(msg_for(press(KeyCode::Char('j'))), Some(Msg::Down));
        assert_eq!(msg_for(press(KeyCode::Char('k'))), Some(Msg::Up));
        assert_eq!(msg_for(press(KeyCode::Enter)), Some(Msg::Enter));
        assert_eq!(msg_for(press(KeyCode::Char('q'))), Some(Msg::Back));
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(msg_for(ctrl_c), Some(Msg::Back));
        let mut release = press(KeyCode::Enter);
        release.kind = KeyEventKind::Release;
        assert_eq!(msg_for(release), None);
    }

    #[test]
    fn rendering_shows_context_selection_and_keys() {
        let model = Model::new(true);
        let screen = render(&model, &Theme::color());
        assert!(screen.contains(text::HOME_TITLE), "{screen}");
        assert!(screen.contains(text::HOME_IN_PROJECT), "{screen}");
        assert!(screen.contains("> Run the development server"), "{screen}");
        assert!(screen.contains("Hot reload while you edit"), "{screen}");
        assert!(screen.contains(text::KEYS_HOME), "{screen}");
    }

    #[test]
    fn monochrome_theme_uses_no_color_and_glyphs_are_ascii() {
        let model = Model::new(false);
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("terminal");
        terminal
            .draw(|frame| view(frame, &model, &Theme::monochrome()))
            .expect("draw");
        let buffer = terminal.backend().buffer();
        let width = buffer.area.width as usize;
        let rows: Vec<String> = buffer
            .content()
            .chunks(width)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect())
            .collect();
        for cell in buffer.content() {
            assert_eq!(cell.fg, Color::Reset);
            assert_eq!(cell.bg, Color::Reset);
        }
        for c in rows.join("\n").chars() {
            let box_drawing = ('\u{2500}'..='\u{257F}').contains(&c);
            assert!(c.is_ascii() || box_drawing, "unexpected glyph {c:?}");
        }
    }
}
