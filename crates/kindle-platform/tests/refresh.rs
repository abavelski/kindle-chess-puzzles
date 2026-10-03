use chess_core::{algebraic_to_square, Board, PieceKind};
use chess_render::{DisplayMetrics, Gray8, Layout, Rect};
use kindle_platform::{
    classify_region, clean_regions_for_board_change, pack_region, plan_present_regions,
    ContentClass, PresentRegion, RefreshMode, RefreshPolicy, RefreshStrength,
};

const METRICS: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn square(name: &str) -> usize {
    algebraic_to_square(name).unwrap()
}

#[test]
fn classifies_both_old_and_new_pixels_and_packs_rows() {
    let old = Gray8::with_stride(4, 3, 6, 255).unwrap();
    let mut new = old.clone();
    new.set_pixel(1, 1, 0);
    let rect = Rect::new(1, 1, 2, 2);
    assert_eq!(
        classify_region(Some(&old), &new, rect),
        ContentClass::Monochrome
    );
    assert_eq!(pack_region(&new, rect).unwrap(), vec![0, 255, 255, 255]);
    new.set_pixel(2, 2, 184);
    assert_eq!(
        classify_region(Some(&new), &old, rect),
        ContentClass::Grayscale
    );
    assert!(pack_region(&new, Rect::new(4, 0, 1, 1)).is_err());
    assert!(pack_region(&new, Rect::new(0, 0, 0, 1)).is_err());
}

#[test]
fn unmeasured_modes_stay_auto_and_verified_modes_are_content_specific() {
    let mut policy = RefreshPolicy::default();
    assert_eq!(
        policy.mode(ContentClass::Monochrome, false),
        RefreshMode::AutoPartial
    );
    assert_eq!(
        policy.mode(ContentClass::Grayscale, false),
        RefreshMode::AutoPartial
    );
    assert_eq!(
        policy.mode(ContentClass::Grayscale, true),
        RefreshMode::Full
    );
    policy.gray_verified = true;
    policy.fast_mono_verified = true;
    assert_eq!(
        policy.mode(ContentClass::Monochrome, false),
        RefreshMode::FastMono
    );
    assert_eq!(
        policy.mode(ContentClass::Grayscale, false),
        RefreshMode::GrayPartial
    );
}

#[test]
fn board_change_planning_targets_only_committed_piece_changes() {
    let layout = Layout::new(METRICS).unwrap();
    let before = Board::starting_position();

    let mut selected = before.clone();
    selected.tap(square("a2"));
    assert!(clean_regions_for_board_change(&before, &selected, false, false, layout).is_empty());

    let mut moved = before.clone();
    moved.tap(square("a2"));
    moved.tap(square("a3"));
    let move_regions = clean_regions_for_board_change(&before, &moved, false, false, layout);
    assert_eq!(move_regions.len(), 2);
    assert!(move_regions.contains(&layout.square_rect(square("a2"))));
    assert!(move_regions.contains(&layout.square_rect(square("a3"))));

    let mut captured = before.clone();
    captured.tap(square("a2"));
    captured.tap(square("a7"));
    let capture_regions = clean_regions_for_board_change(&before, &captured, false, false, layout);
    assert_eq!(capture_regions.len(), 2);
    assert!(capture_regions.contains(&layout.square_rect(square("a2"))));
    assert!(capture_regions.contains(&layout.square_rect(square("a7"))));

    let promo_before = Board::from_fen("8/P7/8/8/8/8/8/8 w - - 0 1").expect("test FEN is valid");
    let mut promoted = promo_before.clone();
    assert!(promoted.promote_pawn(square("a7"), square("a8"), PieceKind::Queen));
    let promotion_regions =
        clean_regions_for_board_change(&promo_before, &promoted, false, false, layout);
    assert_eq!(promotion_regions.len(), 2);

    let mut reply = before.clone();
    reply.tap(square("a2"));
    reply.tap(square("a3"));
    reply.tap(square("h7"));
    reply.tap(square("h6"));
    let reply_regions = clean_regions_for_board_change(&before, &reply, false, false, layout);
    assert_eq!(reply_regions.len(), 4);

    assert!(clean_regions_for_board_change(&before, &before, false, false, layout).is_empty());
}

#[test]
fn large_or_reoriented_board_changes_clean_the_board_once() {
    let layout = Layout::new(METRICS).unwrap();
    let before = Board::starting_position();
    let mut many = before.clone();
    for (from, to) in [("a2", "a3"), ("b2", "b3"), ("c2", "c3")] {
        many.tap(square(from));
        many.tap(square(to));
    }
    assert_eq!(
        clean_regions_for_board_change(&before, &many, false, false, layout),
        vec![layout.board]
    );

    let mut one_move = before.clone();
    one_move.tap(square("a2"));
    one_move.tap(square("a3"));
    assert_eq!(
        clean_regions_for_board_change(&before, &one_move, false, true, layout),
        vec![layout.board]
    );
}

#[test]
fn presentation_plan_replaces_overlapping_partial_damage_with_clean_regions() {
    let viewport = Rect::new(0, 0, 40, 20);
    let clean = Rect::new(5, 0, 5, 10);
    let planned = plan_present_regions(
        &[Rect::new(0, 0, 20, 10), Rect::new(30, 0, 5, 5)],
        &[clean],
        viewport,
    );
    assert!(planned.contains(&PresentRegion::clean(clean)));
    assert!(planned.contains(&PresentRegion::partial(Rect::new(0, 0, 5, 10))));
    assert!(planned.contains(&PresentRegion::partial(Rect::new(10, 0, 10, 10))));
    assert!(planned.contains(&PresentRegion::partial(Rect::new(30, 0, 5, 5))));
    for (index, first) in planned.iter().enumerate() {
        for second in planned.iter().skip(index + 1) {
            assert!(
                !first.rect.intersects(second.rect),
                "planned regions overlap: {first:?} and {second:?}"
            );
        }
    }
    assert_eq!(
        planned.last().map(|region| region.strength),
        Some(RefreshStrength::Clean)
    );
}

#[test]
fn periodic_refresh_counts_successful_updates_only_and_recovery_is_explicit() {
    let mut policy = RefreshPolicy::default();
    policy.full_every = Some(3);
    assert!(!policy.full_due());
    policy.completed(false);
    policy.completed(false);
    assert!(policy.full_due());
    policy.completed(true);
    assert!(!policy.full_due());
    assert_eq!(
        policy.mode(ContentClass::Monochrome, true),
        RefreshMode::Full
    );
    let mut default = RefreshPolicy::default();
    for _ in 0..1000 {
        default.completed(false);
    }
    assert!(!default.full_due());
}
