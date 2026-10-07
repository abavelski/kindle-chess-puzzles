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
fn settings_button_is_immediately_left_of_close_and_opens_the_settings_panel() {
    let app = state();
    let output = render(&app, SCRIBE).expect("render succeeds");
    assert!(output.layout.settings.right() <= output.layout.exit.x);
    assert!(output.layout.collection_button.right() <= output.layout.settings.x);

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
fn settings_panel_matches_files_size_and_anchors_close_at_bottom() {
    let mut app = state();
    app.dispatch(Action::OpenSettings);
    for metrics in [
        SCRIBE,
        DisplayMetrics {
            width: 2480,
            height: 1860,
            dpi: 300,
        },
    ] {
        let output = render(&app, metrics).expect("render succeeds");
        let layout = output.layout;
        assert_eq!(layout.settings_modal, layout.collection_modal);
        let bottom_padding = layout.settings_close.x - layout.settings_modal.x;
        assert_eq!(
            layout.settings_close.bottom() + bottom_padding,
            layout.settings_modal.bottom()
        );
        assert!(layout.settings_notes.bottom() < layout.settings_close.y);
        for rect in [
            layout.settings_free_mode,
            layout.settings_notes,
            layout.settings_close,
        ] {
            assert!(layout.settings_modal.contains_rect(rect));
            assert!(rect.height >= layout.minimum_touch_px());
        }
        let (x, y) = center(layout.settings_close);
        assert_eq!(
            output.hit_test_app(x, y, &app),
            Some(HitTarget::CloseSettings)
        );
        let (x, y) = center(layout.settings_modal);
        assert_eq!(output.hit_test_app(x, y, &app), None);
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
