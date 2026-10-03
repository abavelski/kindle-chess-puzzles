use chess_render::{Gray8, Rect};
use fbink_sys::{FbInkError, RefreshMode};
use kindle_platform::{submit_regions, RefreshPolicy, RegionPresenter};

#[derive(Default)]
struct Mock {
    calls: Vec<&'static str>,
    fail_submission: bool,
}
impl RegionPresenter for Mock {
    fn submit(&mut self, _: Rect, _: &[u8], _: RefreshMode) -> Result<(), FbInkError> {
        self.calls.push("submit");
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
        self.calls.push("wait");
        Ok(())
    }
}

#[test]
fn dirty_batch_submits_all_regions_then_waits_once() {
    let frame = Gray8::new(20, 20, 255);
    let regions = [Rect::new(0, 0, 4, 4), Rect::new(16, 16, 4, 4)];
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
    assert_eq!(backend.calls, ["submit", "submit", "wait"]);
    assert_eq!(timing.regions, 2);
    backend.calls.clear();
    submit_regions(
        &mut backend,
        &frame,
        Some(&frame),
        &[],
        &RefreshPolicy::default(),
        false,
    )
    .unwrap();
    assert!(backend.calls.is_empty());
}

#[test]
fn invalid_batches_and_failed_submissions_do_not_wait_for_a_missing_update() {
    let frame = Gray8::new(20, 20, 255);
    let mut backend = Mock::default();
    assert!(submit_regions(
        &mut backend,
        &frame,
        None,
        &[Rect::new(0, 0, 2, 2), Rect::new(19, 19, 2, 2)],
        &RefreshPolicy::default(),
        false
    )
    .is_err());
    assert!(backend.calls.is_empty());
    backend.fail_submission = true;
    assert!(submit_regions(
        &mut backend,
        &frame,
        None,
        &[Rect::new(0, 0, 2, 2)],
        &RefreshPolicy::default(),
        false
    )
    .is_err());
    assert_eq!(backend.calls, ["submit"]);
}
