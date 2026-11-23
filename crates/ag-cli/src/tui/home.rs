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

use super::project_context::{ProjectContext, StatusSource};
use super::text;
use super::theme::Theme;

/// One menu entry: what it says and the command it launches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Entry {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) command: String,
}

/// Plain data only: no file system, no processes, no clock reads.
#[derive(Debug, Clone)]
pub(crate) struct Model {
    context: &'static str,
    entries: Vec<Entry>,
    selected: usize,
    exists: bool,
    project_name: Option<String>,
    template: Option<String>,
    running: bool,
    pid: Option<u32>,
    bind: Option<String>,
    status_source: StatusSource,
    show_help: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Msg {
    Up,
    Down,
    Enter,
    Back,
    /// Toggle the shortcut legend (pure flag change).
    Help,
    /// Ask the event loop to re-resolve the context (no I/O in the view).
    Refresh,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Effect {
    Launch(String),
    Quit,
    /// A message for the caller, printed after the alternate screen closes.
    Notice(String),
}

/// Builds the exact launcher menu for a context. Entry labels and details are
/// kept byte-for-byte so the frozen regression tests remain valid (QA plan §1).
fn launcher_entries(in_project: bool) -> Vec<Entry> {
    if in_project {
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
    }
}

impl Model {
    /// Compatibility shim: delegates to [`Model::from_project_detected`].
    /// Kept so the nine frozen tests compile and pass unchanged.
    pub(crate) fn new(in_project: bool) -> Model {
        Self::from_project_detected(in_project)
    }

    /// Explicit constructor for callers that only know the markers. The
    /// execution status is always `Unknown` — never assumed running.
    pub(crate) fn from_project_detected(in_project: bool) -> Model {
        Self::from_context(&ProjectContext::detected(in_project))
    }

    /// Real constructor. Consumes a resolved context; no I/O is performed.
    /// Inside a project the `ag new` entry is never offered (D-1).
    pub(crate) fn from_context(ctx: &ProjectContext) -> Model {
        Model {
            context: if ctx.exists {
                text::HOME_IN_PROJECT
            } else {
                text::HOME_OUTSIDE_PROJECT
            },
            entries: launcher_entries(ctx.exists),
            selected: 0,
            exists: ctx.exists,
            project_name: ctx.path.clone(),
            template: ctx.template.clone(),
            running: ctx.running,
            pid: ctx.pid,
            bind: ctx.bind.clone(),
            status_source: ctx.status_source,
            show_help: false,
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
                let entry = self.entries.get(self.selected)?;
                // Defense in depth (D-1): even if a create entry reached the
                // menu inside a project, Enter yields a Notice, never Launch.
                if self.exists && entry.command == "ag new" {
                    return Some(Effect::Notice(text::NOTICE_CREATE_DISABLED.to_owned()));
                }
                return Some(Effect::Launch(entry.command.clone()));
            }
            Msg::Back => return Some(Effect::Quit),
            Msg::Help => {
                self.show_help = !self.show_help;
            }
            Msg::Refresh => {}
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
        KeyCode::Char('?') => Some(Msg::Help),
        KeyCode::Char('r') | KeyCode::F(5) => Some(Msg::Refresh),
        _ => None,
    }
}

/// One-line status summary. PID and bind appear ONLY for a verified real
/// state; `Unknown` never prints a PID or a bind (anti-faking, K-10).
fn status_summary(model: &Model) -> String {
    let name = model.project_name.as_deref().unwrap_or(text::NAME_UNKNOWN);
    let template = model.template.as_deref().unwrap_or(text::TEMPLATE_UNKNOWN);
    let identity = format!(
        "{}: {}   {}: {}",
        text::LABEL_PROJECT,
        name,
        text::LABEL_TEMPLATE,
        template
    );

    if !model.exists {
        return format!(
            "{}   {}: {}",
            identity,
            text::LABEL_STATE,
            text::STATE_NO_PROJECT
        );
    }
    if model.running && model.status_source == StatusSource::Real {
        let pid = model
            .pid
            .map(|value| value.to_string())
            .unwrap_or_else(|| text::NAME_UNKNOWN.to_owned());
        let bind = model.bind.as_deref().unwrap_or(text::NAME_UNKNOWN);
        return format!(
            "{}   {}: {}   {}: {}   {}: {}",
            identity,
            text::LABEL_STATE,
            text::STATE_RUNNING,
            text::LABEL_PID,
            pid,
            text::LABEL_BIND,
            bind
        );
    }
    format!(
        "{}   {}: {}",
        identity,
        text::LABEL_STATE,
        text::STATE_UNKNOWN
    )
}

/// State-specific, actionable hint. No string here contains "running" for
/// the Unknown state, so the anti-faking test can assert its absence.
fn status_hint(model: &Model) -> &'static str {
    if !model.exists {
        text::HINT_NO_PROJECT
    } else if model.running {
        text::HINT_RUNNING
    } else {
        text::HINT_UNKNOWN
    }
}

/// Draws the home screen. Reads the model and nothing else (INV-4).
/// Layout: header, one-line status panel, launcher menu, help footer.
pub(crate) fn view(frame: &mut Frame, model: &Model, theme: &Theme) {
    let [header, status, menu, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(5),
        Constraint::Min(9),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(text::PRODUCT, theme.title),
            Span::raw(format!("  v{}  |  ", env!("CARGO_PKG_VERSION"))),
            Span::styled(text::HOME_TITLE, theme.title),
        ]))
        .block(Block::bordered().border_style(theme.border)),
        header,
    );

    let status_lines = vec![
        Line::from(Span::styled(model.context, theme.label)),
        Line::from(vec![
            Span::styled(text::STATUS_TITLE, theme.label),
            Span::raw("  "),
            Span::styled(status_summary(model), theme.label),
        ]),
        Line::from(Span::styled(status_hint(model), theme.hint)),
    ];
    frame.render_widget(
        Paragraph::new(status_lines)
            .wrap(Wrap { trim: false })
            .block(Block::bordered().border_style(theme.border)),
        status,
    );

    // The `> ` marker stays directly attached to the label so the frozen
    // rendering test ("> Run the development server") keeps passing.
    let mut lines = Vec::new();
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
        menu,
    );

    let keys = if model.show_help {
        text::KEYS_HELP
    } else {
        text::KEYS_HOME
    };
    frame.render_widget(
        Paragraph::new(Span::styled(keys, theme.hint))
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
    use std::time::SystemTime;

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

    // --- New tests (QA plan cases a-h). Synthetic data only (L-6). --------

    fn render_with(model: &Model, theme: &Theme, w: u16, h: u16) -> String {
        let mut terminal = Terminal::new(TestBackend::new(w, h)).expect("terminal");
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

    fn running_context() -> ProjectContext {
        ProjectContext {
            exists: true,
            path: Some("demo-api".into()),
            template: Some("layered".into()),
            running: true,
            pid: Some(4242),
            bind: Some("127.0.0.1:8080".into()),
            status_source: StatusSource::Real,
            detected_at: Some(SystemTime::now()),
        }
    }

    fn unknown_context() -> ProjectContext {
        ProjectContext {
            exists: true,
            path: Some("demo-api".into()),
            template: Some("layered".into()),
            running: false,
            pid: None,
            bind: None,
            status_source: StatusSource::Unknown,
            detected_at: Some(SystemTime::now()),
        }
    }

    #[test]
    fn home_running_project_shows_pid_and_bind() {
        let model = Model::from_context(&running_context());
        let screen = render_with(&model, &Theme::color(), 120, 40);
        assert!(screen.contains(text::STATE_RUNNING), "{screen}");
        assert!(screen.contains("4242"), "{screen}");
        assert!(screen.contains("127.0.0.1:8080"), "{screen}");
        assert!(screen.contains("demo-api"), "{screen}");
        assert!(screen.contains("layered"), "{screen}");
        assert!(!screen.contains("Create a new project"), "{screen}");
        assert!(model.entries.iter().all(|entry| entry.command != "ag new"));
    }

    #[test]
    fn home_unknown_state_never_claims_running() {
        let model = Model::from_context(&unknown_context());
        let screen = render_with(&model, &Theme::color(), 120, 40);
        assert!(screen.contains(text::HOME_IN_PROJECT), "{screen}");
        assert!(screen.contains(text::STATE_UNKNOWN), "{screen}");
        assert!(!screen.contains(text::STATE_RUNNING), "{screen}");
        assert!(!screen.to_ascii_lowercase().contains("running"), "{screen}");
        assert!(!screen.contains("PID: 4242"), "{screen}");
        assert!(!screen.contains("127.0.0.1"), "{screen}");
        assert!(screen.contains("? for help"), "{screen}");
    }

    #[test]
    fn home_status_does_not_show_absolute_paths() {
        let ctx = ProjectContext {
            path: None,
            ..unknown_context()
        };
        let model = Model::from_context(&ctx);
        let screen = render_with(&model, &Theme::color(), 120, 40);
        assert!(screen.contains(text::NAME_UNKNOWN), "{screen}");
        assert!(!screen.contains("/home/"), "{screen}");
        assert!(!screen.contains("/Users/"), "{screen}");
        assert!(!screen.contains("C:\\"), "{screen}");
    }

    #[test]
    fn no_create_entry_inside_project() {
        let model = Model::from_context(&running_context());
        assert!(model.entries.len() >= 4);
        assert!(model.entries.iter().all(|entry| entry.command != "ag new"));
        let screen = render_with(&model, &Theme::color(), 120, 40);
        assert!(!screen.contains("Create a new project"), "{screen}");
    }

    #[test]
    fn create_request_returns_notice_not_launch() {
        let mut model = Model::from_context(&running_context());
        // Simulate a create entry reaching the menu through a race.
        model.entries.push(Entry {
            label: "Create a new project".into(),
            detail: "injected by a race".into(),
            command: "ag new".into(),
        });
        model.selected = model.entries.len() - 1;
        assert_eq!(
            model.update(Msg::Enter),
            Some(Effect::Notice(text::NOTICE_CREATE_DISABLED.into()))
        );
    }

    #[test]
    fn help_and_refresh_are_pure() {
        let mut model = Model::from_context(&unknown_context());
        let selected = model.selected;
        assert_eq!(model.update(Msg::Help), None);
        assert!(model.show_help);
        let screen = render_with(&model, &Theme::color(), 120, 40);
        assert!(screen.contains(text::KEYS_HELP), "{screen}");
        assert_eq!(model.update(Msg::Help), None);
        assert!(!model.show_help);
        assert_eq!(model.update(Msg::Refresh), None);
        assert_eq!(model.selected, selected);
    }

    #[test]
    fn new_keys_are_mapped() {
        assert_eq!(msg_for(press(KeyCode::Char('?'))), Some(Msg::Help));
        assert_eq!(msg_for(press(KeyCode::Char('r'))), Some(Msg::Refresh));
        assert_eq!(msg_for(press(KeyCode::F(5))), Some(Msg::Refresh));
    }

    #[test]
    fn launcher_effects_never_embed_foreign_path() {
        let ctx = ProjectContext {
            path: Some("/tmp/foreign-project".into()),
            ..running_context()
        };
        let mut model = Model::from_context(&ctx);
        for _ in 0..model.entries.len() {
            if let Some(Effect::Launch(command)) = model.update(Msg::Enter) {
                assert!(!command.contains("foreign-project"), "{command}");
            }
            model.update(Msg::Down);
        }
    }

    #[test]
    fn home_monochrome_running_keeps_pid_and_bind() {
        let model = Model::from_context(&running_context());
        let mut terminal = Terminal::new(TestBackend::new(120, 40)).expect("terminal");
        terminal
            .draw(|frame| view(frame, &model, &Theme::monochrome()))
            .expect("draw");
        let buffer = terminal.backend().buffer();
        for cell in buffer.content() {
            assert_eq!(cell.fg, Color::Reset);
            assert_eq!(cell.bg, Color::Reset);
        }
        let screen = render_with(&model, &Theme::monochrome(), 120, 40);
        assert!(screen.contains("4242"), "{screen}");
        assert!(screen.contains("127.0.0.1:8080"), "{screen}");
        assert!(screen.contains(text::STATE_RUNNING), "{screen}");
        for c in screen.chars() {
            let box_drawing = ('\u{2500}'..='\u{257F}').contains(&c);
            assert!(c.is_ascii() || box_drawing, "unexpected glyph {c:?}");
        }
    }
}
