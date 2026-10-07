use chess_core::{
    algebraic_to_square, parse_puzzle_file, Action, ActiveCollection, AppState, CollectionEntry,
    Progress, PromotionChoice,
};
use chess_render::{
    calculate_damage, compact_damage, merge_clipped, render, DisplayMetrics, Gray8, Rect,
};

const METRICS: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};
fn state() -> AppState {
    AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/parity-puzzles.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    )
}
fn tap(name: &str) -> Action {
    Action::TapSquare(algebraic_to_square(name).unwrap())
}

#[test]
fn first_frame_noop_and_geometry_change() {
    let frame = Gray8::new(130, 90, 255);
    assert_eq!(
        calculate_damage(None, &frame),
        vec![Rect::new(0, 0, 130, 90)]
    );
    assert!(calculate_damage(Some(&frame), &frame).is_empty());
    assert_eq!(
        calculate_damage(Some(&Gray8::new(1, 1, 0)), &frame),
        vec![Rect::new(0, 0, 130, 90)]
    );
}

#[test]
fn clips_empty_overflowing_and_merges_transitive_overlaps() {
    let viewport = Rect::new(0, 0, 100, 100);
    let regions = [
        Rect::new(90, 90, u32::MAX, u32::MAX),
        Rect::new(200, 200, 10, 10),
        Rect::new(0, 0, 0, 2),
        Rect::new(0, 0, 10, 10),
        Rect::new(18, 0, 10, 10),
        Rect::new(8, 0, 12, 10),
    ];
    assert_eq!(
        merge_clipped(&regions, viewport),
        vec![Rect::new(0, 0, 28, 10), Rect::new(90, 90, 10, 10)]
    );
}

fn check_transition(app: &mut AppState, action: Action) -> Vec<Rect> {
    let previous = render(app, METRICS).unwrap().frame;
    app.dispatch(action);
    let output = render(app, METRICS).unwrap();
    let current = output.frame;
    let damage = compact_damage(&calculate_damage(Some(&previous), &current), output.layout);
    let mut replay = previous.clone();
    for rect in &damage {
        assert!(Rect::new(0, 0, METRICS.width, METRICS.height).contains_rect(*rect));
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                replay.set_pixel(x as i32, y as i32, current.pixel(x, y).unwrap());
            }
        }
    }
    assert_eq!(
        replay, current,
        "partial presentation must reconstruct every changed pixel"
    );
    damage
}

#[test]
fn all_major_transitions_reconstruct_the_full_render() {
    let mut app = state();
    let selection = check_transition(&mut app, tap("d7"));
    assert!(!selection.is_empty());
    assert!(
        selection.iter().map(|r| r.width * r.height).sum::<u32>()
            < METRICS.width * METRICS.height / 8
    );
    check_transition(&mut app, tap("d7")); // deselect
    check_transition(&mut app, tap("d7"));
    check_transition(&mut app, tap("d8")); // wrong overlay
    check_transition(&mut app, Action::Reset);
    check_transition(&mut app, tap("d7"));
    check_transition(&mut app, tap("e8")); // complete, header, description
    check_transition(&mut app, Action::ToggleDescription);
    check_transition(&mut app, Action::Reset);
    check_transition(&mut app, Action::ToggleMode);
    check_transition(&mut app, tap("f4"));
    check_transition(&mut app, tap("a8")); // two-square move
    check_transition(&mut app, Action::ToggleOrientationLock);
    check_transition(&mut app, Action::Flip);
    check_transition(&mut app, Action::ToggleMode);
    check_transition(&mut app, Action::NextPuzzle);
    check_transition(&mut app, tap("e2"));
    check_transition(&mut app, tap("e6")); // reply and correct hint
    check_transition(&mut app, Action::NextPuzzle);
    check_transition(&mut app, tap("a7"));
    check_transition(&mut app, tap("a8"));
    check_transition(&mut app, Action::CancelPromotion);
    check_transition(&mut app, tap("a7"));
    check_transition(&mut app, tap("a8"));
    check_transition(&mut app, Action::ChoosePromotion(PromotionChoice::Queen));
    app.set_collection_entries(
        (0..10)
            .map(|n| CollectionEntry::valid(format!("puzzles-{n}.json"), format!("Collection {n}")))
            .collect(),
    );
    assert!(!check_transition(&mut app, Action::OpenCollectionPicker).is_empty());
    check_transition(&mut app, Action::CollectionPickerNextPage);
    check_transition(&mut app, Action::CollectionPickerPreviousPage);
    check_transition(&mut app, Action::CloseCollectionPicker);
    check_transition(
        &mut app,
        Action::SetTransientMessage(Some("Storage warning".into())),
    );
    check_transition(&mut app, Action::SetTransientMessage(None));
    check_transition(&mut app, Action::PreviousPuzzle);
}

#[test]
fn moved_piece_origin_is_clean_and_damage_replay_matches_the_new_frame() {
    let mut app = state();
    app.dispatch(Action::ToggleMode);
    app.dispatch(tap("f4"));
    let previous = render(&app, METRICS).unwrap();
    app.dispatch(tap("a8"));
    let current = render(&app, METRICS).unwrap();

    let logical_origin = algebraic_to_square("f4").unwrap();
    let display_origin = if app.flipped() {
        63 - logical_origin
    } else {
        logical_origin
    };
    let origin = current.layout.square_rect(display_origin);
    let row = display_origin / 8;
    let column = display_origin % 8;
    let expected_background = if (row + column) % 2 == 0 { 238 } else { 184 };
    for y in origin.y..origin.bottom() {
        for x in origin.x..origin.right() {
            assert_eq!(
                current.frame.pixel(x, y),
                Some(expected_background),
                "origin square must render as a clean empty cell"
            );
        }
    }

    let damage = compact_damage(
        &calculate_damage(Some(&previous.frame), &current.frame),
        current.layout,
    );
    let mut replay = previous.frame.clone();
    for rect in damage {
        for y in rect.y..rect.bottom() {
            for x in rect.x..rect.right() {
                replay.set_pixel(x as i32, y as i32, current.frame.pixel(x, y).unwrap());
            }
        }
    }
    assert_eq!(replay, current.frame);
}

#[test]
fn stride_padding_is_not_visible_damage_and_edge_tiles_are_clipped() {
    let old = Gray8::with_stride(65, 65, 80, 255).unwrap();
    let mut new = old.clone();
    new.pixels_mut()[79] = 0;
    assert!(calculate_damage(Some(&old), &new).is_empty());
    new.set_pixel(64, 64, 0);
    assert_eq!(
        calculate_damage(Some(&old), &new),
        vec![Rect::new(64, 64, 1, 1)]
    );
}

#[test]
fn scribe_navigation_and_selection_do_not_fragment_into_dozens_of_updates() {
    use chess_render::compact_damage;
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles-endgames.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/puzzles-endgames.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    );
    let old = render(&app, METRICS).unwrap();
    app.dispatch(tap("g6"));
    let selected = render(&app, METRICS).unwrap();
    let damage = compact_damage(
        &calculate_damage(Some(&old.frame), &selected.frame),
        selected.layout,
    );
    assert_eq!(damage.len(), 1, "one square selection needs one refresh");
    assert!(damage[0].width * damage[0].height < METRICS.width * METRICS.height / 8);
    app.dispatch(Action::NextPuzzle);
    let next = render(&app, METRICS).unwrap();
    let damage = compact_damage(
        &calculate_damage(Some(&selected.frame), &next.frame),
        next.layout,
    );
    assert!(
        damage.len() <= 6,
        "navigation must not submit 38 tile fragments: {damage:?}"
    );
    assert!(
        !damage.contains(&next.layout.viewport),
        "navigation stays regional"
    );
}

#[test]
fn board_size_changes_clear_old_geometry_through_partial_damage() {
    let mut app = state();
    check_transition(&mut app, Action::OpenSettings);
    check_transition(&mut app, Action::ToggleBoardSizeSetting);
    check_transition(&mut app, Action::CloseSettings);
    check_transition(&mut app, Action::OpenSettings);
    check_transition(&mut app, Action::ToggleBoardSizeSetting);
    check_transition(&mut app, Action::CloseSettings);
}
