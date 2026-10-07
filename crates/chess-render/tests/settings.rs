use chess_core::{parse_puzzle_file, Action, ActiveCollection, AppState, Progress, Settings};
use chess_render::{render, DisplayMetrics, HitTarget, Layout, ToolbarAlignment};

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
fn default_toolbar_fills_the_available_width_with_fixed_icon_and_stretched_text() {
    let output = render(&state(), SCRIBE).expect("render succeeds");
    let controls = output.layout.toolbar_targets;
    assert_eq!(controls[0].rect.x, output.layout.toolbar.x);
    assert_eq!(controls[5].rect.right(), output.layout.toolbar.right());
    assert_eq!(controls[0].rect.width, output.layout.minimum_touch_px());
    let text_widths: Vec<_> = controls[1..].iter().map(|c| c.rect.width).collect();
    assert!(text_widths
        .iter()
        .all(|width| *width > controls[0].rect.width));
    assert!(text_widths.iter().max().unwrap() - text_widths.iter().min().unwrap() <= 1);
}

#[test]
fn developer_alignment_choice_places_compact_controls_at_either_edge() {
    let app = state();
    let left = Layout::for_app_with_alignment(SCRIBE, &app, ToolbarAlignment::Left).unwrap();
    let right = Layout::for_app_with_alignment(SCRIBE, &app, ToolbarAlignment::Right).unwrap();
    let full = Layout::for_app_with_alignment(SCRIBE, &app, ToolbarAlignment::FullWidth).unwrap();
    assert_eq!(left.toolbar_targets[0].rect.x, left.toolbar.x);
    assert_eq!(right.toolbar_targets[5].rect.right(), right.toolbar.right());
    assert_eq!(
        left.toolbar_targets[0].rect.width,
        right.toolbar_targets[0].rect.width
    );
    for index in 1..6 {
        assert_eq!(
            left.toolbar_targets[index].rect.width,
            right.toolbar_targets[index].rect.width
        );
        assert!(full.toolbar_targets[index].rect.width > left.toolbar_targets[index].rect.width);
    }
    assert!(left.toolbar_targets[5].rect.right() < left.toolbar.right());
    assert!(right.toolbar_targets[0].rect.x > right.toolbar.x);
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
fn settings_header_close_icon_has_the_same_close_action_as_the_bottom_button() {
    let mut app = state();
    app.dispatch(Action::OpenSettings);
    let output = render(&app, SCRIBE).expect("render succeeds");
    let layout = output.layout;
    let icon = layout.settings_close_icon;

    assert!(layout.settings_modal.contains_rect(icon));
    assert_eq!(icon.width, icon.height);
    assert!(icon.width >= layout.minimum_touch_px());
    assert!(icon.x > layout.settings_modal.x + layout.settings_modal.width / 2);
    assert!(icon.bottom() < layout.settings_free_mode.y);

    let (x, y) = center(icon);
    let target = output
        .hit_test_app(x, y, &app)
        .expect("close icon is tappable");
    assert_eq!(target, HitTarget::CloseSettings);
    assert_eq!(target.into_action(), Some(Action::CloseSettings));
    app.dispatch(target.into_action().expect("close target has an action"));
    assert!(!app.settings_open());
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
            layout.settings_close_icon,
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
        let (x, y) = center(layout.settings_close_icon);
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

    assert_eq!(free.rect.width, 0);
    assert_eq!(notes.rect.width, 0);
    let visible: Vec<_> = output
        .layout
        .toolbar_targets
        .iter()
        .filter(|control| control.rect.width > 0)
        .collect();
    assert_eq!(visible.len(), 4);
    assert_eq!(visible.first().unwrap().target, HitTarget::ToggleAnalysis);
    assert_eq!(visible.last().unwrap().target, HitTarget::Flip);
    assert_eq!(
        visible.last().unwrap().rect.right(),
        output.layout.toolbar.right()
    );
    for pair in visible.windows(2) {
        assert!(pair[0].rect.right() < pair[1].rect.x);
    }
    for control in visible {
        let (x, y) = center(control.rect);
        let expected = (control.target != HitTarget::ToggleAnalysis).then_some(control.target);
        assert_eq!(output.hit_test_app(x, y, &app), expected);
    }
}

#[test]
fn each_optional_button_compacts_the_full_width_toolbar() {
    for (toggle, hidden) in [
        (Action::ToggleFreeModeSetting, HitTarget::ToggleMode),
        (Action::ToggleNotesSetting, HitTarget::ToggleDescription),
    ] {
        let mut app = state();
        app.dispatch(Action::OpenSettings);
        app.dispatch(toggle);
        app.dispatch(Action::CloseSettings);
        let output = render(&app, SCRIBE).expect("render succeeds");
        let visible: Vec<_> = output
            .layout
            .toolbar_targets
            .iter()
            .filter(|control| control.rect.width > 0)
            .collect();
        assert_eq!(visible.len(), 5);
        assert_eq!(
            visible.last().unwrap().rect.right(),
            output.layout.toolbar.right()
        );
        for pair in visible.windows(2) {
            assert!(pair[0].rect.right() < pair[1].rect.x);
        }
        assert_eq!(
            output
                .layout
                .toolbar_targets
                .iter()
                .find(|control| control.target == hidden)
                .unwrap()
                .rect
                .width,
            0
        );
        for control in visible
            .into_iter()
            .filter(|c| c.target != HitTarget::ToggleAnalysis)
        {
            let (x, y) = center(control.rect);
            let expected =
                if control.target == HitTarget::ToggleAnalysis && !app.analysis_available() {
                    None
                } else {
                    Some(control.target)
                };
            assert_eq!(output.hit_test_app(x, y, &app), expected);
        }
    }
}

#[test]
fn smaller_board_is_centered_and_reclaims_space_for_full_width_text() {
    let app = state();
    let standard = Layout::for_app(SCRIBE, &app).unwrap();
    let small = AppState::new_with_settings(
        ActiveCollection::from_collection("puzzles.json", parse_puzzle_file(PUZZLES).unwrap()),
        Progress::new(),
        Settings::parse(br#"{"version":1,"small_board":true}"#).unwrap(),
    );
    let layout = Layout::for_app(SCRIBE, &small).unwrap();
    assert_eq!(layout.board.width, standard.board.width * 3 / 5 / 8 * 8);
    assert_eq!(layout.board.y, standard.board.y);
    assert_eq!(layout.board.x, (SCRIBE.width - layout.board.width) / 2);
    assert_eq!(layout.toolbar.width, standard.toolbar.width);
    assert_eq!(
        layout.toolbar.y - layout.board_outer.bottom(),
        standard.toolbar.y - standard.board_outer.bottom()
    );
    assert_eq!(layout.status.width, standard.status.width);
    assert!(layout.status.height > standard.status.height + 600);
}

#[test]
fn board_size_row_toggles_and_small_square_targets_follow_both_orientations() {
    let mut app = state();
    app.dispatch(Action::OpenSettings);
    let output = render(&app, SCRIBE).unwrap();
    let (x, y) = center(output.layout.settings_board_size);
    let target = output.hit_test_app(x, y, &app).unwrap();
    assert_eq!(target, HitTarget::ToggleBoardSizeSetting);
    app.dispatch(target.into_action().unwrap());
    app.dispatch(Action::CloseSettings);
    for metrics in [
        SCRIBE,
        DisplayMetrics {
            width: 2480,
            height: 1860,
            dpi: 300,
        },
    ] {
        for _ in 0..2 {
            let output = render(&app, metrics).unwrap();
            let layout = output.layout;
            for index in 0..64 {
                let (x, y) = center(layout.square_rect(index));
                let logical = if app.flipped() { 63 - index } else { index };
                assert_eq!(
                    output.hit_test_app(x, y, &app),
                    Some(HitTarget::Square(logical))
                );
            }
            for control in layout.toolbar_targets {
                assert!(control.rect.height >= layout.minimum_touch_px());
                assert!(control.rect.width >= layout.minimum_touch_px());
                assert!(!control.rect.intersects(layout.board_outer));
                let (x, y) = center(control.rect);
                let expected =
                    if control.target == HitTarget::ToggleAnalysis && !app.analysis_available() {
                        None
                    } else {
                        Some(control.target)
                    };
                assert_eq!(output.hit_test_app(x, y, &app), expected);
            }
            assert!(layout.status.y >= layout.next.bottom());
            app.dispatch(Action::Flip);
        }
        app.dispatch(Action::OpenSettings);
        let layout = Layout::for_app(metrics, &app).unwrap();
        assert!(layout.settings_board_size.bottom() < layout.settings_close.y);
        assert!(layout
            .settings_modal
            .contains_rect(layout.settings_board_size));
        app.dispatch(Action::CloseSettings);
    }
}
