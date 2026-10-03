use chess_render::{Gray8, Rect};
use fbink_sys::{FbInkError, RefreshMode};
use kindle_platform::{submit_regions, PresentRegion, RefreshPolicy, RegionPresenter};

#[derive(Default)]
struct Mock {
    submissions: Vec<(Rect, RefreshMode)>,
    waits: usize,
    fail_submission: bool,
    pixels: Vec<Vec<u8>>,
}
impl RegionPresenter for Mock {
    fn submit(&mut self, rect: Rect, pixels: &[u8], mode: RefreshMode) -> Result<(), FbInkError> {
        self.submissions.push((rect, mode));
        self.pixels.push(pixels.to_vec());
        if self.fail_submission {
            Err(FbInkError::Call {
                operation: "mock",
                code: -1,
            })
        } else {
            Ok(())
        }
    }
    fn wait(&mut self) -> Result<(), FbInkError> {
        self.waits += 1;
        Ok(())
    }
}

#[test]
fn mixed_batch_submits_partial_and_clean_then_waits_once() {
    let frame = Gray8::new(20, 20, 255);
    let partial = Rect::new(0, 0, 4, 4);
    let clean = Rect::new(16, 16, 4, 4);
    let regions = [PresentRegion::partial(partial), PresentRegion::clean(clean)];
    let mut backend = Mock::default();
    let timing = submit_regions(
        &mut backend,
        &frame,
        Some(&frame),
        &regions,
        &RefreshPolicy::default(),
        false,
    )
    .unwrap();
    assert_eq!(
        backend.submissions,
        [
            (partial, RefreshMode::AutoPartial),
            (clean, RefreshMode::Clean)
        ]
    );
    assert_eq!(backend.waits, 1);
    assert_eq!(timing.regions, 2);

    backend.submissions.clear();
    backend.waits = 0;
    submit_regions(
        &mut backend,
        &frame,
        Some(&frame),
        &[],
        &RefreshPolicy::default(),
        false,
    )
    .unwrap();
    assert!(backend.submissions.is_empty());
    assert_eq!(backend.waits, 0);
}

#[test]
fn whole_screen_clean_uses_existing_full_mode() {
    let frame = Gray8::new(20, 20, 255);
    let viewport = Rect::new(0, 0, 20, 20);
    let mut backend = Mock::default();
    submit_regions(
        &mut backend,
        &frame,
        None,
        &[PresentRegion::clean(viewport)],
        &RefreshPolicy::default(),
        true,
    )
    .unwrap();
    assert_eq!(backend.submissions, [(viewport, RefreshMode::Full)]);
    assert_eq!(backend.waits, 1);
}

#[test]
fn invalid_batches_and_failed_submissions_do_not_wait_for_a_missing_update() {
    let frame = Gray8::new(20, 20, 255);
    let mut backend = Mock::default();
    assert!(submit_regions(
        &mut backend,
        &frame,
        None,
        &[
            PresentRegion::partial(Rect::new(0, 0, 2, 2)),
            PresentRegion::clean(Rect::new(19, 19, 2, 2))
        ],
        &RefreshPolicy::default(),
        false
    )
    .is_err());
    assert!(backend.submissions.is_empty());
    assert_eq!(backend.waits, 0);

    backend.fail_submission = true;
    assert!(submit_regions(
        &mut backend,
        &frame,
        None,
        &[PresentRegion::clean(Rect::new(0, 0, 2, 2))],
        &RefreshPolicy::default(),
        false
    )
    .is_err());
    assert_eq!(backend.submissions.len(), 1);
    assert_eq!(backend.waits, 0);
}

#[test]
fn grayscale_clean_and_startup_submit_final_pixels_without_a_white_pass() {
    for full in [false, true] {
        let frame = Gray8::new(20, 20, 184);
        let rect = if full {
            Rect::new(0, 0, 20, 20)
        } else {
            Rect::new(16, 16, 4, 4)
        };
        let mut backend = Mock::default();
        submit_regions(
            &mut backend,
            &frame,
            None,
            &[PresentRegion::clean(rect)],
            &RefreshPolicy::default(),
            full,
        )
        .unwrap();
        assert_eq!(
            backend.pixels,
            [vec![184; (rect.width * rect.height) as usize]]
        );
        assert_eq!(backend.waits, 1);
    }
}
