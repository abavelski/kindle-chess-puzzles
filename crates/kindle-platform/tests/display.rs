use chess_render::Gray8;
use kindle_platform::{validate_presentable_frame, DisplayState, PresentError};

#[test]
fn display_boundary_rejects_wrong_geometry_before_fbink() {
    let state = DisplayState {
        width: 1860,
        height: 2480,
        scanline_stride: 1872,
        bpp: 8,
        rotation: 3,
        is_y8: true,
    };

    let wrong = Gray8::new(100, 100, 255);
    assert_eq!(
        validate_presentable_frame(&state, &wrong),
        Err(PresentError::GeometryMismatch)
    );

    let right = Gray8::new(1860, 2480, 255);
    assert!(validate_presentable_frame(&state, &right).is_ok());
}

#[test]
fn display_boundary_requires_the_measured_y8_mode() {
    let frame = Gray8::new(1860, 2480, 255);
    let state = DisplayState {
        width: 1860,
        height: 2480,
        scanline_stride: 1872,
        bpp: 16,
        rotation: 3,
        is_y8: false,
    };
    assert_eq!(
        validate_presentable_frame(&state, &frame),
        Err(PresentError::UnsupportedPixelFormat)
    );
}

#[test]
fn invalid_damage_is_rejected_before_any_submission() {
    use chess_render::Rect;
    use kindle_platform::validate_damage;
    let frame = Gray8::new(10, 10, 255);
    assert!(validate_damage(&frame, &[Rect::new(0, 0, 2, 2), Rect::new(9, 9, 2, 2)]).is_err());
    assert!(validate_damage(&frame, &[Rect::new(0, 0, 0, 2)]).is_err());
    assert!(validate_damage(&frame, &[]).is_ok());
}
