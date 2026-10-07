use chess_core::{parse_puzzle_file, ActiveCollection, AppState, Progress};
use chess_render::{calculate_damage, draw_sleeping_overlay, render, DisplayMetrics};

#[test]
fn sleeping_overlay_preserves_board_outside_box_and_progress() {
    let app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    );
    let progress = app.progress().clone();
    let metrics = DisplayMetrics {
        width: 1860,
        height: 2480,
        dpi: 300,
    };
    let awake = render(&app, metrics).unwrap();
    let mut sleeping = awake.clone();
    let rect = draw_sleeping_overlay(&mut sleeping.frame, sleeping.layout, metrics);
    assert!(sleeping.layout.board.contains(rect.x, rect.y));
    assert!(sleeping
        .layout
        .board
        .contains(rect.right() - 1, rect.bottom() - 1));
    assert_eq!(sleeping.frame.pixel(rect.x, rect.y), Some(0));
    assert_eq!(sleeping.frame.pixel(rect.x + 10, rect.y + 10), Some(255));
    for y in 0..metrics.height {
        for x in 0..metrics.width {
            if !rect.contains(x, y) {
                assert_eq!(sleeping.frame.pixel(x, y), awake.frame.pixel(x, y));
            }
        }
    }
    assert!(!calculate_damage(Some(&awake.frame), &sleeping.frame).contains(&awake.layout.viewport));
    assert_eq!(app.progress(), &progress);
    assert_eq!(render(&app, metrics).unwrap().frame, awake.frame);
    if let Ok(path) = std::env::var("SLEEP_SNAPSHOT") {
        std::fs::write(path, sleeping.frame.to_pgm()).unwrap();
    }
    assert_eq!(sleeping.frame.checksum64(), 8_705_426_740_802_654_058);
}
