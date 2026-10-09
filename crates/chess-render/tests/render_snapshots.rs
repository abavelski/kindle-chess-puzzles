use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, CollectionEntry,
    Progress, PromotionChoice,
};
use chess_render::{render, DisplayMetrics};

const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
const PROMOTIONS: &[u8] = include_bytes!("../../../tests/fixtures/promotion-puzzles.json");
const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn square(name: &str) -> usize {
    algebraic_to_square(name).expect("test square")
}

fn state(bytes: &[u8]) -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(bytes).expect("fixture parses"),
        ),
        Progress::new(),
    )
}

fn play(app: &mut AppState, movement: &str) {
    app.dispatch(Action::TapSquare(square(&movement[..2])));
    app.dispatch(Action::TapSquare(square(&movement[2..4])));
    if movement.len() == 5 {
        let choice = match movement.as_bytes()[4] {
            b'q' => PromotionChoice::Queen,
            b'r' => PromotionChoice::Rook,
            b'b' => PromotionChoice::Bishop,
            b'n' => PromotionChoice::Knight,
            _ => panic!("bad promotion"),
        };
        app.dispatch(Action::ChoosePromotion(choice));
    }
}

fn hash(app: &AppState) -> u64 {
    let output = render(app, SCRIBE).expect("render succeeds");
    assert_eq!(output.damage.as_slice(), &[output.layout.viewport]);
    if let Ok(directory) = std::env::var("UI_SNAPSHOT_DIR") {
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            format!("{directory}/{}.pgm", output.frame.checksum64()),
            output.frame.to_pgm(),
        )
        .unwrap();
    }
    output.frame.checksum64()
}

#[test]
fn board_outer_frame_is_thin_gray() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let outer = output.layout.board_outer;

    assert_eq!(frame.pixel(outer.x, outer.y), Some(160));
    assert_eq!(frame.pixel(outer.x + 1, outer.y + 1), Some(160));
    assert_eq!(frame.pixel(outer.x + 2, outer.y + 2), Some(255));
}

#[test]
fn refresh_button_uses_vector_refresh_icon() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let refresh = output
        .layout
        .refresh
        .inset(output.layout.header.height / 10);
    let icon = refresh.inset(refresh.width.min(refresh.height) / 4);
    let left = icon.x + icon.width / 5;
    let right = icon.right().saturating_sub(1) - icon.width / 5;
    let top = icon.y + icon.height / 4;
    let bottom = icon.bottom().saturating_sub(1) - icon.height / 4;
    let mid_x = icon.x + icon.width / 2;
    let mid_y = icon.y + icon.height / 2;

    for (x, y) in [(mid_x, top), (right, mid_y), (mid_x, bottom), (left, mid_y)] {
        assert_eq!(frame.pixel(x, y), Some(0), "refresh icon stroke at {x},{y}");
    }
}

#[test]
fn exit_button_uses_vector_close_icon() {
    let output = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let exit = output.layout.exit.inset(output.layout.header.height / 10);
    let icon = exit.inset(exit.width.min(exit.height) / 4);
    let right = icon.right().saturating_sub(1);
    let bottom = icon.bottom().saturating_sub(1);

    for (x, y) in [
        (icon.x, icon.y),
        (right, icon.y),
        (icon.x, bottom),
        (right, bottom),
    ] {
        assert_eq!(frame.pixel(x, y), Some(0), "close icon corner at {x},{y}");
    }
}

#[test]
fn header_controls_share_compact_height_and_title_is_visible() {
    let mut app = state(PUZZLES);
    play(&mut app, "d7e8");
    app.set_collection_entries(vec![CollectionEntry::valid("puzzles.json", "Puzzles")]);

    let output = render(&app, SCRIBE).expect("render succeeds");
    let frame = output.frame;
    let layout = output.layout;
    let inset = layout.header.height / 10;
    let refresh = layout.refresh.inset(inset);
    let files = layout.collection_button.inset(inset);
    let settings = layout.settings.inset(inset);
    let close = layout.exit.inset(inset);
    let solved_width = layout
        .minimum_touch_px()
        .saturating_mul(2)
        .saturating_sub(inset.saturating_mul(2));
    let solved_height = layout.header.height.saturating_sub(inset.saturating_mul(2));

    assert_eq!(files.width, solved_width);
    assert_eq!(files.height, solved_height);
    assert_eq!(refresh.height, solved_height);
    assert_eq!(settings.height, solved_height);
    assert_eq!(close.height, solved_height);
    assert_eq!(refresh.width, settings.width);
    assert_eq!(settings.width, close.width);
    assert!(close.width < files.width);
    assert!(files.right() <= settings.x);
    assert!(settings.right() <= layout.exit.x);

    let refresh_mid_y = refresh.y + refresh.height / 2;
    assert_eq!(
        frame.pixel(layout.refresh.x + 4, refresh_mid_y),
        Some(255),
        "refresh visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(refresh.x, refresh_mid_y), Some(0));

    let files_mid_y = files.y + files.height / 2;
    assert_eq!(
        frame.pixel(layout.collection_button.x, files_mid_y),
        Some(255),
        "FILES visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(files.x, files_mid_y), Some(0));

    let close_mid_y = close.y + close.height / 2;
    assert_eq!(
        frame.pixel(layout.exit.x, close_mid_y),
        Some(255),
        "close visual should be inset from its full touch target"
    );
    assert_eq!(frame.pixel(close.x, close_mid_y), Some(0));

    let padding = layout.header.height / 8;
    let title_start = layout.refresh.right().saturating_add(padding);
    let title_end = layout
        .collection_button
        .x
        .saturating_sub(layout.minimum_touch_px().saturating_mul(2))
        .saturating_sub(padding);
    let title_ink = (layout.header.y + 6..layout.header.bottom().saturating_sub(6))
        .flat_map(|y| (title_start..title_end).map(move |x| (x, y)))
        .filter(|&(x, y)| matches!(frame.pixel(x, y), Some(tone) if tone < 160))
        .count();
    assert!(title_ink > 20, "expected rendered title text in the header");
}

#[test]
fn status_panel_is_rounded_and_shows_one_large_primary_content() {
    let normal = render(&state(PUZZLES), SCRIBE).expect("render succeeds");
    let status = normal.layout.status;
    let padding = (status.height / 12).max(8);
    let x = status.x + padding;
    let y = status.y + padding;

    assert_eq!(normal.frame.pixel(status.x, status.y), Some(255));
    assert!(
        !(y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(normal.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "side to move now appears in the header"
    );

    let dot = br#"{
      "version":1,
      "puzzles":[{
        "id":"dot-description",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "description":".",
        "solution":["g6g7"]
      }]
    }"#;

    let mut described = state(dot);
    described.dispatch(Action::ToggleDescription);
    let described = render(&described, SCRIBE).expect("render succeeds");
    assert_eq!(described.frame.pixel(x, y), Some(255));
    assert!(
        (y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(described.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "description should render visible text inside the status panel"
    );

    let mut free_described = state(dot);
    free_described.dispatch(Action::ToggleMode);
    free_described.dispatch(Action::ToggleDescription);
    let free_described = render(&free_described, SCRIBE).expect("render succeeds");
    assert_eq!(free_described.frame.pixel(x, y), Some(255));
    assert!(
        (y..status.bottom().saturating_sub(padding)).any(|py| {
            (x..status.right().saturating_sub(padding))
                .any(|px| matches!(free_described.frame.pixel(px, py), Some(tone) if tone < 160))
        }),
        "description text should remain visible in Free Board mode"
    );
}

fn region_has_ink(output: &chess_render::RenderOutput, rect: chess_render::Rect) -> bool {
    (rect.y..rect.bottom())
        .any(|y| (rect.x..rect.right()).any(|x| output.frame.pixel(x, y).unwrap() < 160))
}

#[test]
fn solution_side_is_centered_in_header_and_absent_from_hidden_description() {
    for black in [false, true] {
        let mut document: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
        if black {
            let fen = document["puzzles"][0]["fen"]
                .as_str()
                .unwrap()
                .replace(" w ", " b ");
            document["puzzles"][0]["fen"] = fen.into();
        }
        let mut app = state(&serde_json::to_vec(&document).unwrap());
        let output = render(&app, SCRIBE).unwrap();
        let center = chess_render::Rect::new(
            SCRIBE.width / 2 - 140,
            10,
            280,
            output.layout.header.height - 20,
        );
        assert!(
            region_has_ink(&output, center),
            "side label should be in header center"
        );
        assert!(
            !region_has_ink(&output, output.layout.status.inset(30)),
            "hidden description should not duplicate side label"
        );
        app.dispatch(Action::Flip);
        let flipped = render(&app, SCRIBE).unwrap();
        for y in center.y..center.bottom() {
            for x in center.x..center.right() {
                assert_eq!(
                    output.frame.pixel(x, y),
                    flipped.frame.pixel(x, y),
                    "Flip must not change solver side"
                );
            }
        }
    }
}

#[test]
fn topic_is_default_solution_status_and_note_contains_only_description() {
    let mut document: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
    document["puzzles"][0]["topic"] =
        serde_json::json!("Геометрический мотив\nКоневые вилки\nУстранение защиты");
    document["puzzles"][0]["description"] =
        serde_json::json!("A description instead of the topic.");
    let mut app = state(&serde_json::to_vec(&document).unwrap());
    let before_progress = app.progress().clone();
    let normal = render(&app, SCRIBE).unwrap();
    let rect = normal.layout.status;
    let content = rect.inset((rect.height / 12).max(8));
    for line in 0..3 {
        assert!(
            region_has_ink(
                &normal,
                chess_render::Rect::new(content.x, content.y + line * 61, content.width, 61)
            ),
            "missing larger topic line {line}"
        );
    }
    assert!(!app.description_visible());
    app.dispatch(Action::ToggleDescription);
    let note = render(&app, SCRIBE).unwrap();
    document["puzzles"][0]["topic"] = serde_json::Value::Null;
    let mut without_topic = state(&serde_json::to_vec(&document).unwrap());
    without_topic.dispatch(Action::ToggleDescription);
    assert_eq!(
        note.frame,
        render(&without_topic, SCRIBE).unwrap().frame,
        "NOTE must contain description only"
    );
    app.dispatch(Action::ToggleDescription);
    assert_eq!(render(&app, SCRIBE).unwrap().frame, normal.frame);
    assert_eq!(app.progress(), &before_progress);
    app.dispatch(Action::Reset);
    assert_eq!(render(&app, SCRIBE).unwrap().frame, normal.frame);
    app.dispatch(Action::NextPuzzle);
    assert!(
        !region_has_ink(&render(&app, SCRIBE).unwrap(), content),
        "topic must not leak into another puzzle"
    );
    app.dispatch(Action::PreviousPuzzle);
    assert_eq!(render(&app, SCRIBE).unwrap().frame, normal.frame);
    play(&mut app, "d7e8");
    assert!(app.description_visible());
    app.dispatch(Action::ToggleDescription);
    assert!(
        region_has_ink(&render(&app, SCRIBE).unwrap(), content),
        "topic returns after hiding solved note"
    );
}

#[test]
fn parity_visual_states_match_reviewed_gray8_snapshots() {
    let mut actual = Vec::new();

    let white = state(PUZZLES);
    actual.push(("white", hash(&white)));

    let mut black = state(PUZZLES);
    black.dispatch(Action::NextPuzzle);
    actual.push(("black", hash(&black)));

    let mut selected = state(PUZZLES);
    selected.dispatch(Action::TapSquare(square("d7")));
    actual.push(("selected", hash(&selected)));

    let mut correct = state(PUZZLES);
    correct.dispatch(Action::NextPuzzle);
    play(&mut correct, "e2e6");
    actual.push(("correct", hash(&correct)));

    let mut wrong = state(PUZZLES);
    play(&mut wrong, "d7d8");
    actual.push(("wrong", hash(&wrong)));

    let mut complete = state(PUZZLES);
    play(&mut complete, "d7e8");
    actual.push(("complete", hash(&complete)));

    let mut solved = complete.clone();
    solved.dispatch(Action::Reset);
    actual.push(("solved", hash(&solved)));

    let mut free = state(PUZZLES);
    free.dispatch(Action::ToggleMode);
    actual.push(("free-board", hash(&free)));

    let mut locked = state(PUZZLES);
    locked.dispatch(Action::ToggleOrientationLock);
    actual.push(("orientation-lock", hash(&locked)));

    let mut described = state(PUZZLES);
    described.dispatch(Action::ToggleDescription);
    actual.push(("description", hash(&described)));

    let mut promotion = state(PROMOTIONS);
    promotion.dispatch(Action::TapSquare(square("a7")));
    promotion.dispatch(Action::TapSquare(square("a8")));
    actual.push(("promotion", hash(&promotion)));

    let long = br#"{
      "version":1,
      "puzzles":[{
        "id":"long-description",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "description":"A long first line explaining the tactical idea.\nA second line wraps across the deterministic description region so clipping is visible.",
        "difficulty":"Hard",
        "solution":["g6g7"]
      }]
    }"#;
    let mut long_description = state(long);
    long_description.dispatch(Action::ToggleDescription);
    actual.push(("long-description", hash(&long_description)));

    let number = br#"{
      "version":1,
      "puzzles":[{
        "id":"number-difficulty",
        "fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1",
        "difficulty":2.5,
        "solution":["g6g7"]
      }]
    }"#;
    actual.push(("number-difficulty", hash(&state(number))));

    let mut settings = state(PUZZLES);
    settings.dispatch(Action::OpenSettings);
    actual.push(("settings-panel", hash(&settings)));

    settings.dispatch(Action::ToggleFreeModeSetting);
    settings.dispatch(Action::ToggleNotesSetting);
    settings.dispatch(Action::CloseSettings);
    actual.push(("toolbar-both-hidden", hash(&settings)));

    let mut picker = state(PUZZLES);
    picker.set_collection_entries(vec![
        CollectionEntry::valid("puzzles.json", "Lichess sample puzzles"),
        CollectionEntry::valid("puzzles-endgames.json", "Endgames"),
    ]);
    picker.dispatch(Action::OpenCollectionPicker);
    actual.push(("collection-picker", hash(&picker)));

    let mut puzzle_goto = state(PROMOTIONS);
    puzzle_goto.dispatch(Action::OpenPuzzleGoto);
    puzzle_goto.dispatch(Action::PuzzleGotoDigit(3));
    actual.push(("puzzle-goto", hash(&puzzle_goto)));

    let mut picker_error = state(PUZZLES);
    picker_error.set_collection_entries(vec![
        CollectionEntry::valid("puzzles.json", "Lichess sample puzzles"),
        CollectionEntry::invalid("puzzles-broken.json", "Invalid progress fixture"),
    ]);
    picker_error.dispatch(Action::OpenCollectionPicker);
    actual.push(("collection-picker-error", hash(&picker_error)));

    let mut warning = state(PUZZLES);
    warning.dispatch(Action::SetTransientMessage(Some(
        "Progress warning: write failed; newest progress remains in memory.".to_owned(),
    )));
    actual.push(("progress-warning", hash(&warning)));

    let mut topic_json: serde_json::Value = serde_json::from_slice(PUZZLES).unwrap();
    topic_json["puzzles"][0]["topic"] =
        serde_json::json!("Геометрический мотив\nКоневые вилки\nУстранение защиты");
    let mut topic = state(&serde_json::to_vec(&topic_json).unwrap());
    actual.push(("topic-default", hash(&topic)));
    let mut black_topic_json = topic_json.clone();
    black_topic_json["puzzles"][0]["fen"] =
        serde_json::json!("8/3B2pp/p5k1/6P1/1ppp1K2/8/1P6/8 b - - 0 39");
    actual.push((
        "black-side-header",
        hash(&state(&serde_json::to_vec(&black_topic_json).unwrap())),
    ));
    topic.dispatch(Action::ToggleDescription);
    actual.push(("topic-note", hash(&topic)));
    topic.dispatch(Action::ToggleDescription);
    play(&mut topic, "d7d8");
    actual.push(("topic-wrong", hash(&topic)));
    topic.dispatch(Action::Reset);
    actual.push(("topic-reset", hash(&topic)));
    topic.dispatch(Action::ToggleMode);
    topic.dispatch(Action::ToggleDescription);
    actual.push(("topic-free-board", hash(&topic)));
    topic_json["puzzles"][0]["description"] = serde_json::Value::Null;
    let mut topic_only = state(&serde_json::to_vec(&topic_json).unwrap());
    topic_only.dispatch(Action::ToggleDescription);
    actual.push(("topic-only", hash(&topic_only)));
    topic_json["puzzles"][0]["id"] = serde_json::json!(
        "an-extremely-long-puzzle-identifier-that-must-not-overlap-the-centered-side-label"
    );
    actual.push((
        "long-header",
        hash(&state(&serde_json::to_vec(&topic_json).unwrap())),
    ));

    const EXPECTED: &[(&str, u64)] = &[
        ("white", 9_760_894_365_916_440_813),
        ("black", 8_593_948_898_781_614_127),
        ("selected", 1_725_143_829_086_474_077),
        ("correct", 9_372_062_701_321_505_223),
        ("wrong", 17_041_585_568_168_551_804),
        ("complete", 8_093_415_275_269_334_814),
        ("solved", 3_368_849_747_325_092_352),
        ("free-board", 3_658_783_538_781_271_082),
        ("orientation-lock", 9_561_028_451_284_079_869),
        ("description", 17_360_303_566_641_089_540),
        ("promotion", 8_157_208_443_279_174_683),
        ("long-description", 115_374_366_990_050_245),
        ("number-difficulty", 8_302_406_461_767_038_921),
        ("settings-panel", 15_015_938_234_012_989_282),
        ("toolbar-both-hidden", 4_733_193_552_348_731_552),
        ("collection-picker", 863_362_585_217_474_590),
        ("puzzle-goto", 7_868_128_827_776_782_836),
        ("collection-picker-error", 17_237_771_526_867_793_393),
        ("progress-warning", 11_028_304_019_650_871_434),
        ("topic-default", 17_521_169_582_287_760_842),
        ("black-side-header", 2_504_381_279_138_994_476),
        ("topic-note", 17_360_303_566_641_089_540),
        ("topic-wrong", 16_286_056_173_196_779_967),
        ("topic-reset", 17_521_169_582_287_760_842),
        ("topic-free-board", 10_787_182_737_743_285_232),
        ("topic-only", 2_339_285_739_454_535_546),
        ("long-header", 8_313_023_483_803_530_321),
    ];

    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn smaller_board_visual_states_match_reviewed_gray8_snapshots() {
    fn small(bytes: &[u8]) -> AppState {
        let mut app = state(bytes);
        app.dispatch(Action::OpenSettings);
        app.dispatch(Action::ToggleBoardSizeSetting);
        app.dispatch(Action::CloseSettings);
        app
    }
    let mut actual = Vec::new();
    let mut app = small(PUZZLES);
    actual.push(("small-board", hash(&app)));
    app.dispatch(Action::ToggleDescription);
    actual.push(("small-description", hash(&app)));
    app.dispatch(Action::OpenSettings);
    actual.push(("small-settings", hash(&app)));
    app.dispatch(Action::CloseSettings);
    app.dispatch(Action::OpenPuzzleGoto);
    actual.push(("small-goto", hash(&app)));
    app.dispatch(Action::CancelPuzzleGoto);
    app.set_collection_entries(vec![CollectionEntry::valid("puzzles.json", "Puzzles")]);
    app.dispatch(Action::OpenCollectionPicker);
    actual.push(("small-collections", hash(&app)));
    let mut promotion = small(PROMOTIONS);
    play(&mut promotion, "a7a8");
    assert!(promotion.pending_promotion().is_some());
    actual.push(("small-promotion", hash(&promotion)));
    let rich = include_bytes!("../../../tests/fixtures/rich-analysis/valid-rich.json");
    let mut analysis = small(rich);
    analysis.dispatch(Action::ToggleAnalysis);
    actual.push(("small-analysis", hash(&analysis)));
    let mut topic = small(r#"{"version":1,"puzzles":[{"id":"topic","fen":"7k/8/5KQ1/8/8/8/8/8 w - - 0 1","solution":["g6g7"],"topic":"Find the winning move.\nНайдите лучший ход."}]}"#.as_bytes());
    actual.push(("small-topic", hash(&topic)));
    topic.dispatch(Action::Flip);
    actual.push(("small-flipped", hash(&topic)));
    let mut free = small(PUZZLES);
    free.dispatch(Action::ToggleMode);
    actual.push(("small-free", hash(&free)));
    free.dispatch(Action::ToggleOrientationLock);
    actual.push(("small-locked", hash(&free)));
    free.dispatch(Action::ToggleFreeModeSetting);
    free.dispatch(Action::ToggleNotesSetting);
    actual.push(("small-hidden-controls", hash(&free)));
    const EXPECTED: &[(&str, u64)] = &[
        ("small-board", 12993960575876375483),
        ("small-description", 14227947985857266275),
        ("small-settings", 1061144063131778047),
        ("small-goto", 12019553447108263684),
        ("small-collections", 2167974840530297815),
        ("small-promotion", 7602527400203273217),
        ("small-analysis", 7743292542008109140),
        ("small-topic", 14699384574776479462),
        ("small-flipped", 7664561400626763616),
        ("small-free", 17613509344666808717),
        ("small-locked", 4512497070776115097),
        ("small-hidden-controls", 5675597048798315616),
    ];
    assert_eq!(actual.as_slice(), EXPECTED);
}

#[test]
fn lock_flip_visibility_states_match_reviewed_snapshots() {
    let directory = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/lock-flip-snapshots");
    std::fs::create_dir_all(&directory).unwrap();
    let mut actual = Vec::new();
    for review in [false, true] {
        for small in [false, true] {
            for mask in 0..4 {
                let mut app = state(PUZZLES);
                if review {
                    let games = chess_core::parse_review_file(include_bytes!(
                        "../../../tests/fixtures/game-review/valid-standard.json"
                    ))
                    .unwrap();
                    app.set_review_games(vec![chess_core::ReviewGameEntry::new(
                        "games.json",
                        games.games[0].clone(),
                    )]);
                    app.dispatch(Action::ToggleWorkspace);
                }
                if small {
                    app.dispatch(Action::ToggleBoardSizeSetting);
                }
                app.dispatch(Action::OpenSettings);
                if mask & 1 != 0 {
                    app.dispatch(Action::ToggleLockSetting);
                }
                if mask & 2 != 0 {
                    app.dispatch(Action::ToggleFlipSetting);
                }
                for panel in [true, false] {
                    if !panel {
                        app.dispatch(Action::CloseSettings);
                    }
                    let name = format!(
                        "{}-{}-{}-{mask}",
                        if review { "review" } else { "puzzles" },
                        if small { "small" } else { "standard" },
                        if panel { "settings" } else { "toolbar" }
                    );
                    let output = render(&app, SCRIBE).unwrap();
                    std::fs::write(directory.join(format!("{name}.pgm")), output.frame.to_pgm())
                        .unwrap();
                    actual.push((name, output.frame.checksum64()));
                }
            }
        }
    }
    let expected: &[(&str, u64)] = &[
        ("puzzles-standard-settings-0", 15015938234012989282),
        ("puzzles-standard-toolbar-0", 9760894365916440813),
        ("puzzles-standard-settings-1", 15136196115950374479),
        ("puzzles-standard-toolbar-1", 5368491620130335297),
        ("puzzles-standard-settings-2", 8815529355947867728),
        ("puzzles-standard-toolbar-2", 14164218220769922806),
        ("puzzles-standard-settings-3", 2333362797031225879),
        ("puzzles-standard-toolbar-3", 16380937667021888876),
        ("puzzles-small-settings-0", 1934371648860011403),
        ("puzzles-small-toolbar-0", 12993960575876375483),
        ("puzzles-small-settings-1", 12865716518698441506),
        ("puzzles-small-toolbar-1", 3898325412899183329),
        ("puzzles-small-settings-2", 9461441995957935514),
        ("puzzles-small-toolbar-2", 8356509320562856687),
        ("puzzles-small-settings-3", 17844612672892500155),
        ("puzzles-small-toolbar-3", 2324479933444545465),
        ("review-standard-settings-0", 1254089386655541435),
        ("review-standard-toolbar-0", 11072394659977528714),
        ("review-standard-settings-1", 14688834136394427084),
        ("review-standard-toolbar-1", 11225645094069730444),
        ("review-standard-settings-2", 5941027920095007879),
        ("review-standard-toolbar-2", 1906972547681482743),
        ("review-standard-settings-3", 15809082168483943088),
        ("review-standard-toolbar-3", 12086145632678130161),
        ("review-small-settings-0", 146955315280021403),
        ("review-small-toolbar-0", 16894878170182868405),
        ("review-small-settings-1", 17645953705525726290),
        ("review-small-toolbar-1", 2460953561994008959),
        ("review-small-settings-2", 16773773237708738530),
        ("review-small-toolbar-2", 5058954955722275157),
        ("review-small-settings-3", 4726954423383144179),
        ("review-small-toolbar-3", 16514494898803383399),
    ];
    let actual: Vec<_> = actual
        .iter()
        .map(|(name, checksum)| (name.as_str(), *checksum))
        .collect();
    assert_eq!(actual.as_slice(), expected);
}
