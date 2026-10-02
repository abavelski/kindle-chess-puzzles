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
