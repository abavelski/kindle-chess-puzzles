use chess_render::{Gray8, Rect};

#[test]
fn canvas_clips_rectangle_operations_and_respects_stride() {
    let mut canvas = Gray8::with_stride(4, 3, 6, 255).expect("valid stride");
    canvas.fill_rect(Rect::new(2, 1, 5, 5), 40);
    canvas.stroke_rect(Rect::new(0, 0, 4, 3), 1, 0);

    assert_eq!(canvas.pixel(0, 0), Some(0));
    assert_eq!(canvas.pixel(3, 2), Some(0));
    assert_eq!(canvas.pixel(2, 1), Some(40));
    assert_eq!(canvas.pixels().len(), 18);
    assert_eq!(&canvas.pixels()[4..6], &[255, 255]);
}

#[test]
fn alpha_blit_is_integer_deterministic_and_clipped() {
    let mut canvas = Gray8::new(3, 2, 200);
    let alpha = [0_u8, 128, 255, 255];
    canvas.blit_alpha(2, 2, 2, &alpha, (1, 0), 0);

    assert_eq!(canvas.pixel(1, 0), Some(200));
    assert_eq!(canvas.pixel(2, 0), Some(100));
    assert_eq!(canvas.pixel(1, 1), Some(0));
    assert_eq!(canvas.pixel(2, 1), Some(0));
}

#[test]
fn gray_blit_uses_source_stride_and_canvas_clipping() {
    let mut canvas = Gray8::new(3, 2, 255);
    let source = [10_u8, 20, 99, 30, 40, 99];
    canvas.blit_gray(2, 2, 3, &source, 2, 0);

    assert_eq!(canvas.pixel(2, 0), Some(10));
    assert_eq!(canvas.pixel(2, 1), Some(30));
}
