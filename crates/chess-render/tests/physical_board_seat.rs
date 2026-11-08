use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, Effect, PhysicalBoardSeat as Seat,
    Progress,
};
use chess_render::{render, DisplayMetrics, HitTarget};
fn app() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!("../../../tests/fixtures/puzzles.json")).unwrap(),
        ),
        Progress::new(),
    )
}
#[test]
fn all_seats_have_touch_sized_settings_rows_and_reviewed_snapshots() {
    let mut actual = Vec::new();
    for small in [false, true] {
        for landscape in [false, true] {
            let metrics = DisplayMetrics {
                width: if landscape { 2480 } else { 1860 },
                height: if landscape { 1860 } else { 2480 },
                dpi: 300,
            };
            let mut app = app();
            if small {
                app.dispatch(Action::ToggleBoardSizeSetting);
            }
            app.dispatch(Action::OpenSettings);
            for seat in [Seat::White, Seat::Black, Seat::Changing] {
                let before = render(&app, metrics).unwrap();
                let row = before.layout.settings_physical_board;
                let hit = before.hit_test_app(row.x + row.width / 2, row.y + row.height / 2, &app);
                assert_eq!(hit, Some(HitTarget::CyclePhysicalBoardSeat));
                assert_eq!(
                    app.dispatch(hit.unwrap().into_action().unwrap()),
                    vec![Effect::SettingsChanged]
                );
                assert_eq!(app.physical_board_seat(), seat);
                let out = render(&app, metrics).unwrap();
                let rows = [
                    out.layout.settings_free_mode,
                    out.layout.settings_notes,
                    out.layout.settings_lock,
                    out.layout.settings_flip,
                    out.layout.settings_board_size,
                    out.layout.settings_physical_board,
                    out.layout.settings_chessnut,
                    out.layout.settings_close,
                ];
                for (i, row) in rows.iter().enumerate() {
                    assert!(out.layout.settings_modal.contains_rect(*row));
                    assert!(row.height >= out.layout.minimum_touch_px());
                    for other in &rows[i + 1..] {
                        assert!(!row.intersects(*other));
                    }
                }
                let name = format!(
                    "{}-{}-{}",
                    if small { "small" } else { "standard" },
                    if landscape { "landscape" } else { "portrait" },
                    seat.label()
                );
                let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../target/physical-board-seat-snapshots");
                std::fs::create_dir_all(&folder).unwrap();
                std::fs::write(
                    folder.join(format!("{}.pgm", out.frame.checksum64())),
                    out.frame.to_pgm(),
                )
                .unwrap();
                actual.push((name, out.frame.checksum64()));
            }
        }
    }
    let expected = [
        ("standard-portrait-WHITE SIDE", 17766412599997071302u64),
        ("standard-portrait-BLACK SIDE", 14757477694213866066u64),
        ("standard-portrait-CHANGING SIDE", 10641348192551124477u64),
        ("standard-landscape-WHITE SIDE", 16283935323773702122u64),
        ("standard-landscape-BLACK SIDE", 16151771513127141162u64),
        ("standard-landscape-CHANGING SIDE", 11829810389413836309u64),
        ("small-portrait-WHITE SIDE", 12713508052790443475u64),
        ("small-portrait-BLACK SIDE", 3607176008153408395u64),
        ("small-portrait-CHANGING SIDE", 7770809778059619036u64),
        ("small-landscape-WHITE SIDE", 7214239419314377401u64),
        ("small-landscape-BLACK SIDE", 10721419626640051645u64),
        ("small-landscape-CHANGING SIDE", 16140375829421402562u64),
    ]
    .into_iter()
    .map(|(name, hash)| (name.to_owned(), hash))
    .collect::<Vec<_>>();
    assert_eq!(actual, expected);
}
