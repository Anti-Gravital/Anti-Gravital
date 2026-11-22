//! Tests of the project creation wizard: transitions, key mapping, rendering.

use super::*;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;

fn free(_: &str) -> bool {
    false
}

fn taken(_: &str) -> bool {
    true
}

fn catalog_template(name: &str) -> Template {
    Template::parse(name).expect("cataloged template")
}

fn type_name(wizard: &mut Wizard, name: &str) {
    for c in name.chars() {
        assert_eq!(wizard.update(Msg::Char(c)), None);
    }
}

fn render(wizard: &Wizard, theme: &Theme, width: u16, height: u16) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("terminal");
    terminal
        .draw(|frame| view(frame, wizard, theme))
        .expect("draw");
    terminal
}

fn screen_text(terminal: &Terminal<TestBackend>) -> String {
    let buffer = terminal.backend().buffer();
    let width = buffer.area.width as usize;
    buffer
        .content()
        .chunks(width)
        .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

// ---- State transitions -------------------------------------------------

#[test]
fn happy_path_submits_the_typed_request() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "my-api");
    assert_eq!(wizard.update(Msg::Enter), None);
    assert_eq!(wizard.step(), Step::Template);
    assert_eq!(wizard.update(Msg::Down), None);
    assert_eq!(wizard.update(Msg::Enter), None);
    assert_eq!(wizard.step(), Step::Review);
    assert_eq!(
        wizard.update(Msg::Enter),
        Some(Effect::Submit(NewProjectRequest {
            name: "my-api".into(),
            template: catalog_template("realtime"),
            modules: Vec::new(),
        }))
    );
}

#[test]
fn empty_name_cannot_advance() {
    let mut wizard = Wizard::new(None, None, free);
    assert_eq!(wizard.update(Msg::Enter), None);
    assert_eq!(wizard.step(), Step::Name);
    assert!(wizard.error.is_some());
}

#[test]
fn invalid_character_is_reported_while_typing_and_blocks_enter() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "my api");
    assert!(wizard.error.is_some());
    assert_eq!(wizard.update(Msg::Enter), None);
    assert_eq!(wizard.step(), Step::Name);
}

#[test]
fn backspace_clears_the_error_once_the_name_is_valid_again() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "api!");
    assert!(wizard.error.is_some());
    wizard.update(Msg::Backspace);
    assert_eq!(wizard.error, None);
    assert_eq!(wizard.name, "api");
}

#[test]
fn existing_directory_is_rejected() {
    let mut wizard = Wizard::new(None, None, taken);
    type_name(&mut wizard, "exists");
    wizard.update(Msg::Enter);
    assert_eq!(wizard.step(), Step::Name);
    assert_eq!(wizard.error.as_deref(), Some(text::NAME_TAKEN));
}

#[test]
fn name_length_is_bounded() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, &"a".repeat(MAX_NAME_LENGTH + 20));
    assert_eq!(wizard.name.len(), MAX_NAME_LENGTH);
}

#[test]
fn back_navigates_to_the_previous_step_and_keeps_the_input() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "shop");
    wizard.update(Msg::Enter);
    wizard.update(Msg::Down);
    wizard.update(Msg::Down);
    wizard.update(Msg::Enter);
    assert_eq!(wizard.step(), Step::Review);
    assert_eq!(wizard.update(Msg::Back), None);
    assert_eq!(wizard.step(), Step::Template);
    assert_eq!(wizard.update(Msg::Back), None);
    assert_eq!(wizard.step(), Step::Name);
    assert_eq!(wizard.name, "shop");
    assert_eq!(wizard.request().template, catalog_template("fullstack"));
}

#[test]
fn back_on_the_first_step_cancels() {
    let mut wizard = Wizard::new(None, None, free);
    assert_eq!(wizard.update(Msg::Back), Some(Effect::Quit));
}

#[test]
fn abort_cancels_from_every_step() {
    for steps_forward in 0..3 {
        let mut wizard = Wizard::new(None, None, free);
        type_name(&mut wizard, "x");
        for _ in 0..steps_forward {
            wizard.update(Msg::Enter);
        }
        assert_eq!(wizard.update(Msg::Abort), Some(Effect::Quit));
    }
}

#[test]
fn selection_stays_within_the_list() {
    let mut wizard = Wizard::new(Some("x".into()), None, free);
    assert_eq!(wizard.step(), Step::Template);
    wizard.update(Msg::Up);
    assert_eq!(wizard.request().template, catalog_template("rest"));
    // Walk past the end of a long catalog; the selection must clamp, not wrap.
    for _ in 0..Template::ALL.len() + 10 {
        wizard.update(Msg::Down);
    }
    let last = Template::ALL.last().expect("catalog is not empty");
    assert_eq!(wizard.request().template, *last);
}

#[test]
fn prefilled_inputs_are_kept() {
    let wizard = Wizard::new(
        Some("given".into()),
        Some(catalog_template("fullstack")),
        free,
    );
    assert_eq!(wizard.step(), Step::Template);
    assert_eq!(
        wizard.request(),
        NewProjectRequest {
            name: "given".into(),
            template: catalog_template("fullstack"),
            modules: Vec::new(),
        }
    );
}

#[test]
fn invalid_prefilled_name_starts_on_the_name_step_with_the_error() {
    let wizard = Wizard::new(Some("bad name".into()), None, free);
    assert_eq!(wizard.step(), Step::Name);
    assert!(wizard.error.is_some());
}

#[test]
fn directory_created_during_review_sends_the_user_back_to_the_name() {
    let mut wizard = Wizard::new(Some("race".into()), None, free);
    wizard.update(Msg::Enter);
    assert_eq!(wizard.step(), Step::Review);
    wizard.name_taken = taken;
    assert_eq!(wizard.update(Msg::Enter), None);
    assert_eq!(wizard.step(), Step::Name);
    assert_eq!(wizard.error.as_deref(), Some(text::NAME_TAKEN));
}

// ---- Key mapping -------------------------------------------------------

fn press(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn letters_are_text_on_the_name_step_and_commands_elsewhere() {
    assert_eq!(
        msg_for(Step::Name, press(KeyCode::Char('q'))),
        Some(Msg::Char('q'))
    );
    assert_eq!(
        msg_for(Step::Template, press(KeyCode::Char('q'))),
        Some(Msg::Abort)
    );
    assert_eq!(
        msg_for(Step::Review, press(KeyCode::Char('b'))),
        Some(Msg::Back)
    );
    assert_eq!(
        msg_for(Step::Template, press(KeyCode::Char('j'))),
        Some(Msg::Down)
    );
}

#[test]
fn control_c_always_aborts() {
    for step in [Step::Name, Step::Template, Step::Review] {
        let key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(msg_for(step, key), Some(Msg::Abort));
    }
}

#[test]
fn key_releases_are_ignored() {
    let mut key = press(KeyCode::Enter);
    key.kind = KeyEventKind::Release;
    assert_eq!(msg_for(Step::Name, key), None);
}

// ---- Rendering ---------------------------------------------------------

#[test]
fn name_step_renders_field_hint_and_keys() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "my-api");
    for (width, height) in [(80, 24), (120, 40)] {
        let screen = screen_text(&render(&wizard, &Theme::color(), width, height));
        assert!(screen.contains("Step 1 of 3: Name"), "{screen}");
        assert!(screen.contains("> my-api_"), "{screen}");
        assert!(screen.contains(text::NAME_HINT), "{screen}");
        assert!(screen.contains(text::KEYS_NAME), "{screen}");
    }
}

#[test]
fn name_error_is_rendered_with_a_text_prefix() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "a b");
    let screen = screen_text(&render(&wizard, &Theme::monochrome(), 80, 24));
    assert!(screen.contains(text::ERROR_PREFIX), "{screen}");
}

#[test]
fn template_step_marks_the_selection_and_lists_its_files() {
    let mut wizard = Wizard::new(Some("shop".into()), None, free);
    wizard.update(Msg::Down);
    wizard.update(Msg::Down);
    // The fourth template adds two rendered lines; a taller frame keeps the
    // full file list visible so this test checks content, not clipping.
    for (width, height) in [(80, 40), (120, 40)] {
        let screen = screen_text(&render(&wizard, &Theme::color(), width, height));
        assert!(screen.contains("Step 2 of 3: Template"), "{screen}");
        assert!(screen.contains("> fullstack"), "{screen}");
        assert!(screen.contains("  rest"), "{screen}");
        assert!(screen.contains("migrations/0001_init.sql"), "{screen}");
    }
}

#[test]
fn review_step_shows_exactly_what_will_be_created_and_the_command() {
    let mut wizard = Wizard::new(
        Some("shop".into()),
        Some(catalog_template("fullstack")),
        free,
    );
    wizard.update(Msg::Enter);
    for (width, height) in [(80, 24), (120, 40)] {
        let screen = screen_text(&render(&wizard, &Theme::color(), width, height));
        assert!(screen.contains("Step 3 of 3: Review"), "{screen}");
        assert!(screen.contains("Directory: ./shop"), "{screen}");
        for file in catalog_template("fullstack").files() {
            assert!(screen.contains(&format!("shop/{file}")), "{screen}");
        }
        assert!(
            screen.contains("ag new shop --template fullstack"),
            "{screen}"
        );
        assert!(screen.contains(text::REVIEW_NOTHING_WRITTEN), "{screen}");
    }
}

#[test]
fn monochrome_theme_uses_no_color_on_any_screen() {
    let mut wizard = Wizard::new(None, None, free);
    type_name(&mut wizard, "a b");
    let mut screens = vec![render(&wizard, &Theme::monochrome(), 80, 24)];
    let mut wizard = Wizard::new(Some("ok".into()), None, free);
    screens.push(render(&wizard, &Theme::monochrome(), 80, 24));
    wizard.update(Msg::Enter);
    screens.push(render(&wizard, &Theme::monochrome(), 80, 24));
    for terminal in &screens {
        for cell in terminal.backend().buffer().content() {
            assert_eq!(cell.fg, Color::Reset);
            assert_eq!(cell.bg, Color::Reset);
        }
    }
}

#[test]
fn every_screen_is_ascii_or_box_drawing_only() {
    let mut wizard = Wizard::new(Some("ok".into()), None, free);
    let mut screens = vec![screen_text(&render(&wizard, &Theme::color(), 80, 24))];
    wizard.update(Msg::Enter);
    screens.push(screen_text(&render(&wizard, &Theme::color(), 80, 24)));
    wizard.update(Msg::Back);
    wizard.update(Msg::Back);
    screens.push(screen_text(&render(&wizard, &Theme::color(), 80, 24)));
    for screen in screens {
        for c in screen.chars() {
            let box_drawing = ('\u{2500}'..='\u{257F}').contains(&c);
            assert!(c.is_ascii() || box_drawing, "unexpected glyph {c:?}");
        }
    }
}

// ---- Long catalog handling ---------------------------------------------

#[test]
fn template_window_follows_the_selection_and_stays_in_bounds() {
    let total = Template::ALL.len();
    assert!(total > MAX_VISIBLE_TEMPLATES);

    assert_eq!(template_window(0, total), (0, MAX_VISIBLE_TEMPLATES));
    let middle = template_window(total / 2, total);
    assert!(middle.0 > 0 && middle.1 < total);
    assert_eq!(middle.1 - middle.0, MAX_VISIBLE_TEMPLATES);

    let last = template_window(total - 1, total);
    assert_eq!(last.1, total);
    assert_eq!(last.1 - last.0, MAX_VISIBLE_TEMPLATES);

    // An out-of-range selection is clamped and never produces an invalid range.
    assert_eq!(template_window(usize::MAX, total), last);
    assert_eq!(template_window(0, 0), (0, 0));
    let short = template_window(0, 4);
    assert_eq!(short, (0, 4));
}

#[test]
fn every_catalog_name_can_be_reached_with_down() {
    let mut wizard = Wizard::new(Some("x".into()), None, free);
    assert_eq!(wizard.step(), Step::Template);
    for (index, expected) in Template::ALL.iter().enumerate() {
        assert_eq!(
            wizard.request().template,
            *expected,
            "selection {index} differs from the catalog"
        );
        if index + 1 < Template::ALL.len() {
            wizard.update(Msg::Down);
        }
    }
}

#[test]
fn a_long_list_renders_scroll_hints_and_the_position() {
    let total = Template::ALL.len();

    let mut bottom = Wizard::new(Some("x".into()), None, free);
    for _ in 0..total - 1 {
        bottom.update(Msg::Down);
    }
    let screen = screen_text(&render(&bottom, &Theme::color(), 80, 24));
    assert!(
        screen.contains(&format!("Template {total} of {total}")),
        "{screen}"
    );
    assert!(screen.contains(text::TEMPLATE_MORE_ABOVE), "{screen}");
    assert!(screen.contains("testing-suite"), "{screen}");
    assert!(!screen.contains("static-site"), "{screen}");

    let middle = Wizard::new(
        Some("x".into()),
        Some(catalog_template("realtime-chat-v2")),
        free,
    );
    let screen = screen_text(&render(&middle, &Theme::color(), 100, 30));
    assert!(screen.contains(text::TEMPLATE_MORE_ABOVE), "{screen}");
    assert!(screen.contains(text::TEMPLATE_MORE_BELOW), "{screen}");
    assert!(screen.contains("> realtime-chat-v2"), "{screen}");
}

#[test]
fn the_selected_template_is_visible_on_a_small_terminal() {
    let mut wizard = Wizard::new(Some("x".into()), None, free);
    for _ in 0..Template::ALL.len() - 1 {
        wizard.update(Msg::Down);
    }
    let screen = screen_text(&render(&wizard, &Theme::color(), 80, 24));
    assert!(screen.contains("testing-suite"), "{screen}");
}
