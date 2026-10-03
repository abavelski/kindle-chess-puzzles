use fbink_sys::{validate_frame, validate_rect, ValidationError};

#[test]
fn frame_validation_rejects_zero_dimensions_short_buffers_and_stride_mismatch() {
    assert_eq!(validate_frame(0, 2480, 0), Err(ValidationError::EmptyFrame));
    assert_eq!(validate_frame(1860, 0, 0), Err(ValidationError::EmptyFrame));
    assert_eq!(
        validate_frame(1860, 2480, 1860 * 2480 - 1),
        Err(ValidationError::BufferLength)
    );
    assert!(validate_frame(1860, 2480, 1860 * 2480).is_ok());
}

#[test]
fn rectangle_validation_rejects_empty_and_out_of_bounds_regions() {
    assert_eq!(
        validate_rect(0, 0, 0, 10, 1860, 2480),
        Err(ValidationError::EmptyRect)
    );
    assert_eq!(
        validate_rect(1850, 2470, 20, 20, 1860, 2480),
        Err(ValidationError::RectOutOfBounds)
    );
    assert!(validate_rect(0, 0, 1860, 2480, 1860, 2480).is_ok());
}

#[test]
fn dimensions_must_fit_the_fbink_signed_integer_api() {
    let too_large = u32::try_from(i32::MAX).unwrap() + 1;
    assert_eq!(
        validate_frame(too_large, 1, usize::try_from(too_large).unwrap()),
        Err(ValidationError::DimensionTooLarge)
    );
}

#[test]
fn region_offsets_fit_fbinks_signed_short_coordinates() {
    use fbink_sys::validate_offset;
    assert!(validate_offset(32767, 32767).is_ok());
    assert!(validate_offset(32768, 0).is_err());
    assert!(validate_offset(0, u32::MAX).is_err());
}
