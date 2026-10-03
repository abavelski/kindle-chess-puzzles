use chess_render::{Gray8, Rect};
use kindle_platform::{classify_region, pack_region, ContentClass, RefreshMode, RefreshPolicy};

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
