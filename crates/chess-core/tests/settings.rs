use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, BoardMode, Effect, Progress, Settings,
};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");

fn state(settings: Settings) -> AppState {
    AppState::new_with_settings(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(PUZZLES).expect("fixture parses"),
        ),
        Progress::new(),
        settings,
    )
}

#[test]
fn settings_default_round_trip_and_reject_future_versions() {
    let settings = Settings::default();
    assert!(settings.show_free_mode_button());
    assert!(settings.show_notes_button());

    let bytes = settings.to_bytes().expect("encode settings");
    assert_eq!(Settings::parse(&bytes).expect("parse settings"), settings);
    assert!(Settings::parse(
        br#"{"version":2,"show_free_mode_button":true,"show_notes_button":true}"#
    )
    .is_err());
}

#[test]
fn settings_panel_is_modal_and_toggles_emit_persistence_effects() {
    let mut app = state(Settings::default());
    app.dispatch(Action::ToggleMode);
    assert_eq!(app.mode(), BoardMode::FreeBoard);

    app.dispatch(Action::OpenSettings);
    assert!(app.settings_open());

    let board = app.board().clone();
    app.dispatch(Action::TapSquare(0));
    assert_eq!(
        app.board(),
        &board,
        "settings blocks ordinary board actions"
    );

    assert_eq!(
        app.dispatch(Action::ToggleFreeModeSetting),
        vec![Effect::SettingsChanged]
    );
    assert!(!app.settings().show_free_mode_button());
    assert_eq!(
        app.mode(),
        BoardMode::Solution,
        "hiding the free-mode control must not strand the app in free mode"
    );

    assert_eq!(
        app.dispatch(Action::ToggleNotesSetting),
        vec![Effect::SettingsChanged]
    );
    assert!(!app.settings().show_notes_button());

    app.dispatch(Action::CloseSettings);
    assert!(!app.settings_open());
    app.dispatch(Action::ToggleMode);
    assert_eq!(app.mode(), BoardMode::Solution);
    app.dispatch(Action::ToggleDescription);
    assert!(!app.description_visible());
}

#[test]
fn disabling_notes_hides_an_already_visible_note() {
    let mut app = state(Settings::default());
    app.dispatch(Action::ToggleDescription);
    assert!(app.description_visible());

    app.dispatch(Action::OpenSettings);
    app.dispatch(Action::ToggleNotesSetting);

    assert!(!app.settings().show_notes_button());
    assert!(!app.description_visible());
}

#[test]
fn smaller_board_setting_survives_serialization() {
    let settings = Settings::parse(br#"{"version":1,"small_board":true}"#).unwrap();
    let encoded = String::from_utf8(settings.to_bytes().unwrap()).unwrap();
    assert!(encoded.contains("\"small_board\":true"));
}

#[test]
fn board_size_toggle_preserves_puzzle_and_preview_and_defaults_for_old_settings() {
    assert!(!Settings::default().small_board());
    assert!(!Settings::parse(br#"{"version":1}"#).unwrap().small_board());
    assert!(Settings::parse(br#"{"version":1,"small_board":"small"}"#).is_err());
    let rich = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
    let mut app = AppState::new(
        ActiveCollection::from_collection("rich.json", parse_puzzle_file(rich).unwrap()),
        Progress::new(),
    );
    app.dispatch(Action::ToggleAnalysis);
    let node = app
        .active_puzzle()
        .analysis
        .as_ref()
        .unwrap()
        .root()
        .children[0];
    app.dispatch(Action::SelectAnalysisNode(node));
    let board = app.board().clone();
    let ply = app.solution_ply();
    let selected = app.selected_analysis_node();
    let progress = app.progress().clone();
    let feedback = app.feedback();
    app.dispatch(Action::OpenSettings);
    for expected in [true, false] {
        assert_eq!(
            app.dispatch(Action::ToggleBoardSizeSetting),
            vec![Effect::SettingsChanged]
        );
        assert_eq!(app.settings().small_board(), expected);
        assert_eq!(app.board(), &board);
        assert_eq!(app.progress(), &progress);
        assert_eq!(app.feedback(), feedback);
        assert_eq!(app.solution_ply(), ply);
        assert_eq!(app.selected_analysis_node(), selected);
        assert!(app.analysis_browser_open());
        assert!(app.settings_open());
    }
}

fn add_review(app: &mut AppState) {
    let collection = chess_core::parse_review_file(include_bytes!(
        "../../../tests/fixtures/game-review/valid-standard.json"
    ))
    .unwrap();
    app.set_review_games(vec![chess_core::ReviewGameEntry::new(
        "games-standard.json",
        collection.games[0].clone(),
    )]);
}

#[test]
fn workspace_settings_are_independent_in_both_directions() {
    let mut app = state(Settings::default());
    add_review(&mut app);
    let original = app.settings().clone();
    app.dispatch(Action::ToggleWorkspace);
    for action in [
        Action::ToggleFreeModeSetting,
        Action::ToggleNotesSetting,
        Action::ToggleBoardSizeSetting,
    ] {
        assert_eq!(app.dispatch(action), vec![Effect::SettingsChanged]);
    }
    let review = app.settings().clone();
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(
        app.settings(),
        &original,
        "review edits must not affect puzzle settings"
    );
    app.dispatch(Action::ToggleBoardSizeSetting);
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(
        app.settings(),
        &review,
        "puzzle edits must not affect review settings"
    );
}

#[test]
fn disabling_review_controls_preserves_inactive_puzzle_free_board_and_notes() {
    let mut app = state(Settings::default());
    add_review(&mut app);
    app.dispatch(Action::ToggleMode);
    app.dispatch(Action::ToggleDescription);
    let board = app.board().clone();
    let progress = app.progress().clone();
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ReviewToggleFree);
    app.dispatch(Action::ToggleFreeModeSetting);
    app.dispatch(Action::ToggleNotesSetting);
    assert!(!app.review_state().unwrap().free_board_enabled());
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.mode(), BoardMode::FreeBoard);
    assert!(app.description_visible());
    assert_eq!(app.board(), &board);
    assert_eq!(app.progress(), &progress);
}

#[test]
fn legacy_preferences_initialize_both_workspaces_and_distinct_profiles_survive_restart() {
    let settings = Settings::parse(br#"{"version":1,"small_board":true,"show_free_mode_button":false,"show_notes_button":false}"#).unwrap();
    let mut app = state(settings);
    add_review(&mut app);
    let legacy = app.settings().clone();
    app.dispatch(Action::ToggleWorkspace);
    assert_eq!(app.settings(), &legacy);
    app.dispatch(Action::ToggleBoardSizeSetting);
    app.dispatch(Action::ToggleFreeModeSetting);
    app.dispatch(Action::ToggleNotesSetting);
    let review = app.settings().clone();
    let mut restored =
        state(Settings::parse(&app.persisted_settings().to_bytes().unwrap()).unwrap());
    assert_eq!(restored.settings(), &legacy);
    add_review(&mut restored);
    restored.dispatch(Action::ToggleWorkspace);
    assert_eq!(restored.settings(), &review);
    restored.dispatch(Action::ToggleWorkspace);
    restored.dispatch(Action::ToggleNotesSetting);
    let puzzle = restored.settings().clone();
    let mut again =
        state(Settings::parse(&restored.persisted_settings().to_bytes().unwrap()).unwrap());
    assert_eq!(again.settings(), &puzzle);
    add_review(&mut again);
    again.dispatch(Action::ToggleWorkspace);
    assert_eq!(again.settings(), &review);
}

#[test]
fn hiding_puzzle_free_control_preserves_paused_review_scratch_session() {
    let mut app = state(Settings::default());
    add_review(&mut app);
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ReviewToggleFree);
    app.dispatch(Action::ToggleWorkspace);
    app.dispatch(Action::ToggleFreeModeSetting);
    app.dispatch(Action::ToggleWorkspace);
    assert!(app.settings().show_free_mode_button());
    assert!(app.review_state().unwrap().free_board_enabled());
}

#[test]
fn malformed_review_preferences_are_rejected() {
    assert!(Settings::parse(br#"{"version":1,"game_review":{"small_board":"small"}}"#).is_err());
}
