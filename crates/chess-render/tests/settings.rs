use chess_core::{parse_puzzle_file, Action, ActiveCollection, AppState, Progress, Settings};
use chess_render::{render, DisplayMetrics, HitTarget};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn state() -> AppState {
    AppState::new_with_settings(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(PUZZLES).expect("fixture parses"),
        ),
        Progress::new(),
        Settings::default(),
    )
}

fn center(rect: chess_render::Rect) -> (u32, u32) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}

#[test]
fn settings_button_is_left_of_refresh_and_opens_the_settings_panel() {
    let app = state();
    let output = render(&app, SCRIBE).expect("render succeeds");
    assert!(output.layout.settings.right() <= output.layout.refresh.x);

    let (x, y) = center(output.layout.settings);
    assert_eq!(
        output.hit_test_app(x, y, &app),
        Some(HitTarget::OpenSettings)
    );
}

#[test]
fn settings_modal_exposes_free_mode_notes_and_close_targets() {
    let mut app = state();
    app.dispatch(Action::OpenSettings);
    let output = render(&app, SCRIBE).expect("render succeeds");

    for (rect, target) in [
        (
            output.layout.settings_free_mode,
            HitTarget::ToggleFreeModeSetting,
        ),
        (output.layout.settings_notes, HitTarget::ToggleNotesSetting),
        (output.layout.settings_close, HitTarget::CloseSettings),
    ] {
        let (x, y) = center(rect);
        assert_eq!(output.hit_test_app(x, y, &app), Some(target));
    }
}

#[test]
fn hidden_toolbar_controls_are_not_hit_targets() {
    let mut app = state();
    app.dispatch(Action::OpenSettings);
    app.dispatch(Action::ToggleFreeModeSetting);
    app.dispatch(Action::ToggleNotesSetting);
    app.dispatch(Action::CloseSettings);

    let output = render(&app, SCRIBE).expect("render succeeds");
    let free = output
        .layout
        .toolbar_targets
        .iter()
        .find(|target| target.target == HitTarget::ToggleMode)
        .expect("free target");
    let notes = output
        .layout
        .toolbar_targets
        .iter()
        .find(|target| target.target == HitTarget::ToggleDescription)
        .expect("notes target");

    for rect in [free.rect, notes.rect] {
        let (x, y) = center(rect);
        assert_eq!(output.hit_test_app(x, y, &app), None);
    }
}
