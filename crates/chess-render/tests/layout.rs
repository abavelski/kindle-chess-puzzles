use chess_core::{
    parse_puzzle_file, Action, ActiveCollection, AppState, CollectionEntry, Progress,
};
use chess_render::{DisplayMetrics, HitTarget, Layout, Rect, MIN_TOUCH_MM};

const SCRIBE: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn center(rect: Rect) -> (u32, u32) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}

#[test]
fn scribe_layout_is_square_prominent_and_inside_viewport() {
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");

    assert_eq!(layout.board.width, layout.board.height);
    assert_eq!(layout.board.width % 8, 0);
    assert!(layout.board.width <= SCRIBE.width * 9 / 10);
    assert!(layout.board.width > SCRIBE.width * 3 / 4);
    assert!(layout.viewport.contains_rect(layout.board));
    assert!(layout.viewport.contains_rect(layout.header));
    assert!(layout.viewport.contains_rect(layout.status));
    assert!(!layout.board.intersects(layout.toolbar));
    assert!(!layout.board.intersects(layout.previous));
    assert!(!layout.board.intersects(layout.next));
}

#[test]
fn controls_and_promotion_targets_meet_minimum_physical_size() {
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");
    let minimum = layout.minimum_touch_px();
    assert_eq!(minimum, ((SCRIBE.dpi * MIN_TOUCH_MM * 10 + 127) / 254));

    for target in layout.toolbar_targets {
        assert!(target.rect.width >= minimum);
        assert!(target.rect.height >= minimum);
    }
    for rect in [layout.previous, layout.next, layout.promotion_cancel] {
        assert!(rect.width >= minimum);
        assert!(rect.height >= minimum);
    }
    for rect in layout.promotion_choices {
        assert!(rect.width >= minimum);
        assert!(rect.height >= minimum);
    }
}

#[test]
fn all_64_display_squares_hit_the_same_logical_square_or_reversed_when_flipped() {
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");

    for display_square in 0..64 {
        let (x, y) = center(layout.square_rect(display_square));
        assert_eq!(
            layout.hit_test(x, y, false, false),
            Some(HitTarget::Square(display_square))
        );
        assert_eq!(
            layout.hit_test(x, y, true, false),
            Some(HitTarget::Square(63 - display_square))
        );
    }
}

#[test]
fn promotion_modal_captures_only_modal_targets() {
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");
    let (board_x, board_y) = center(layout.square_rect(0));
    assert_eq!(layout.hit_test(board_x, board_y, false, true), None);

    for (index, rect) in layout.promotion_choices.into_iter().enumerate() {
        let (x, y) = center(rect);
        assert_eq!(
            layout.hit_test(x, y, false, true),
            Some(HitTarget::Promotion(index.try_into().expect("0..4")))
        );
    }

    let (x, y) = center(layout.promotion_cancel);
    assert_eq!(
        layout.hit_test(x, y, false, true),
        Some(HitTarget::CancelPromotion)
    );
}

#[test]
fn landscape_metrics_still_produce_valid_layout_without_magic_scribe_coordinates() {
    let landscape = DisplayMetrics {
        width: 2480,
        height: 1860,
        dpi: 300,
    };
    let layout = Layout::new(landscape).expect("landscape metrics fit");
    assert_eq!(layout.board.width, layout.board.height);
    assert!(layout.viewport.contains_rect(layout.board));
    assert!(layout.status.bottom() <= landscape.height);
}

#[test]
fn collection_picker_targets_are_touch_sized_and_block_the_board() {
    const PUZZLES: &[u8] = include_bytes!("../../../tests/fixtures/puzzles.json");
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(PUZZLES).expect("fixture parses"),
        ),
        Progress::new(),
    );
    app.set_collection_entries(
        (0..7)
            .map(|index| {
                CollectionEntry::valid(
                    format!("puzzles-{index}.json"),
                    format!("Collection {index}"),
                )
            })
            .collect(),
    );

    for rect in layout.collection_rows {
        assert!(rect.width >= layout.minimum_touch_px());
        assert!(rect.height >= layout.minimum_touch_px());
    }
    for rect in [
        layout.collection_page_previous,
        layout.collection_page_next,
        layout.collection_close,
    ] {
        assert!(rect.width >= layout.minimum_touch_px());
        assert!(rect.height >= layout.minimum_touch_px());
    }

    let (x, y) = center(layout.collection_button);
    assert_eq!(
        layout.hit_test_app(x, y, &app),
        Some(HitTarget::OpenCollections)
    );
    app.dispatch(Action::OpenCollectionPicker);

    let (x, y) = center(layout.collection_rows[0]);
    assert_eq!(
        layout.hit_test_app(x, y, &app),
        Some(HitTarget::Collection(0))
    );
    let (board_x, board_y) = center(layout.square_rect(0));
    assert_eq!(layout.hit_test_app(board_x, board_y, &app), None);

    let (x, y) = center(layout.collection_page_next);
    assert_eq!(
        layout.hit_test_app(x, y, &app),
        Some(HitTarget::CollectionNextPage)
    );
    app.dispatch(Action::CollectionPickerNextPage);
    let (x, y) = center(layout.collection_rows[0]);
    assert_eq!(
        layout.hit_test_app(x, y, &app),
        Some(HitTarget::Collection(6))
    );
}

#[test]
fn header_edge_controls_are_touch_sized_and_accessible_during_modals() {
    let layout = Layout::new(SCRIBE).expect("Scribe layout");
    for rect in [layout.refresh, layout.exit] {
        assert!(rect.width >= layout.minimum_touch_px());
        assert!(rect.height >= layout.minimum_touch_px());
    }
    for rect in [
        layout.board_outer,
        layout.previous,
        layout.next,
        layout.status,
    ] {
        assert!(!layout.exit.intersects(rect));
    }
    assert_eq!(layout.refresh.x, layout.header.x);
    assert_eq!(layout.refresh.y, layout.header.y);
    assert_eq!(layout.exit.right(), layout.header.right());
    assert_eq!(layout.exit.y, layout.header.y);
    assert!(!layout.refresh.intersects(layout.exit));
    assert!(!layout.exit.intersects(layout.collection_button));

    let (refresh_x, refresh_y) = center(layout.refresh);
    assert_eq!(
        layout.hit_test(refresh_x, refresh_y, false, true),
        Some(HitTarget::Refresh)
    );
    let (x, y) = center(layout.exit);
    assert_eq!(layout.hit_test(x, y, false, true), Some(HitTarget::Exit));
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/promotion-puzzles.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    );
    assert_eq!(layout.hit_test_app(x, y, &app), Some(HitTarget::Exit));
    assert_eq!(
        layout.hit_test_app(refresh_x, refresh_y, &app),
        Some(HitTarget::Refresh)
    );
    app.set_collection_entries(vec![CollectionEntry::valid("puzzles.json", "Puzzles")]);
    app.dispatch(Action::OpenCollectionPicker);
    assert_eq!(layout.hit_test_app(x, y, &app), Some(HitTarget::Exit));
    assert_eq!(
        layout.hit_test_app(refresh_x, refresh_y, &app),
        Some(HitTarget::Refresh)
    );
    assert_eq!(HitTarget::Exit.into_action(), Some(Action::Exit));
    assert_eq!(HitTarget::Refresh.into_action(), None);
}

#[test]
fn scribe_header_compact_controls_and_status_prioritize_description_space() {
    let layout = Layout::new(SCRIBE).expect("Scribe metrics fit");

    assert_eq!(layout.header.x, 0);
    assert_eq!(layout.header.y, 0);
    assert_eq!(layout.header.right(), SCRIBE.width);
    assert!(!layout.header.intersects(layout.board_outer));

    for target in layout.toolbar_targets {
        let visual = layout.control_visual_rect(target.rect);
        assert!(target.rect.height >= layout.minimum_touch_px());
        assert!(visual.height < target.rect.height);
        assert_eq!(visual.height, layout.toolbar.height);
    }

    for target in [layout.previous, layout.next] {
        let visual = layout.control_visual_rect(target);
        assert!(target.height >= layout.minimum_touch_px());
        assert!(visual.height < target.height);
        assert_eq!(visual.height, layout.toolbar.height);
    }

    assert!(layout.board.y < 200);
    assert!(layout.status.height >= 280);
}
