//! Shared display/input/power/presentation loop for Kindle products.
use chess_render::{
    calculate_damage, compact_damage, draw_sleeping_overlay, render, DisplayMetrics, HitTarget,
    ScreenState,
};
use kindle_platform::{
    DeviceEvent, KindleDisplay, PowerEvent, PowerEvents, RefreshPolicy, ScribeInput, TapPolicy,
    SCRIBE_DPI,
};
use std::time::Instant;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Poll {
    Idle,
    Changed,
    Exit,
}
pub trait Application {
    fn screen(&self) -> &impl ScreenState;
    fn poll(&mut self) -> Poll {
        Poll::Idle
    }
    fn tap(&mut self, target: HitTarget) -> bool;
    fn pause_input(&mut self) {}
    fn flush(&mut self);
}
pub fn run(
    mut application: impl Application,
    process_start: Instant,
) -> Result<(), Box<dyn std::error::Error>> {
    let _framebuffer_owner = FramebufferOwner::from_supervisor()?;
    let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    for signal in [
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
        signal_hook::consts::SIGHUP,
    ] {
        signal_hook::flag::register(signal, shutdown.clone())?;
    }
    let force_full = std::env::args().skip(1).any(|arg| arg == "--full-refresh");
    let mut display = KindleDisplay::open()?;
    eprintln!(
        "kindle-chess: timing fbink_initialized={}ms",
        process_start.elapsed().as_millis()
    );
    let display_state = display.state();
    let metrics = DisplayMetrics {
        width: display_state.width,
        height: display_state.height,
        dpi: SCRIBE_DPI,
    };
    let mut input = ScribeInput::discover(metrics, TapPolicy::scribe_default())?;
    input.take_exclusive()?;
    eprintln!("kindle-chess: exclusive finger input acquired before first frame");

    eprintln!(
        "kindle-chess: FBInk {} at {}x{} stride={} bpp={} rotation={}",
        display.fbink_version()?,
        display_state.width,
        display_state.height,
        display_state.scanline_stride,
        display_state.bpp,
        display_state.rotation
    );
    for diagnostic in input.diagnostics() {
        eprintln!("kindle-chess: input candidate {diagnostic}");
    }
    eprintln!(
        "kindle-chess: selected /dev/input/{} ({})",
        input.selected().event_name,
        input.selected().name
    );
    eprintln!(
        "kindle-chess: selected pen /dev/input/{} ({})",
        input.selected_pen().event_name,
        input.selected_pen().name
    );
    let mut power = PowerEvents::open()?;
    let mut sleeping = false;
    let mut previous = None;
    let mut policy = RefreshPolicy::default();
    let mut touch_received: Option<Instant> = None;
    let mut clean_regions = Vec::new();
    let mut force_full_next = false;
    loop {
        let mut output = render(application.screen(), metrics)?;
        if sleeping {
            draw_sleeping_overlay(&mut output.frame, output.layout, metrics);
        }
        output.damage = compact_damage(
            &calculate_damage(previous.as_ref(), &output.frame),
            output.layout,
        );
        let timing = display.present_damage(
            &output.frame,
            previous.as_ref(),
            &output.damage,
            &clean_regions,
            &mut policy,
            force_full || force_full_next,
        )?;
        force_full_next = false;
        if previous.is_none() {
            eprintln!(
                "kindle-chess: timing first_usable_frame={}ms",
                process_start.elapsed().as_millis()
            );
        }
        if timing.regions > 0 {
            let touch_to_submit = touch_received.map(|start| {
                start
                    .elapsed()
                    .saturating_sub(timing.complete)
                    .saturating_add(timing.submit)
            });
            eprintln!("kindle-chess: timing regions={} full={} submit={}ms complete={}ms recognized_touch_to_submit={:?}", timing.regions, timing.full, timing.submit.as_millis(), timing.complete.as_millis(), touch_to_submit);
        }
        previous = Some(output.frame.clone());
        clean_regions.clear();
        let event = loop {
            if shutdown.load(std::sync::atomic::Ordering::Relaxed) {
                application.flush();
                return Ok(());
            }
            let before = application.screen().board().clone();
            let flipped_before = application.screen().flipped();
            match application.poll() {
                Poll::Exit => {
                    application.flush();
                    return Ok(());
                }
                Poll::Changed => {
                    clean_regions.extend(crate::clean_regions_for_board_change(
                        &before,
                        application.screen().board(),
                        flipped_before,
                        application.screen().flipped(),
                        output.layout,
                    ));
                    break None;
                }
                Poll::Idle => {}
            }
            if let Some(event) = input.poll_event(&mut power, 100)? {
                break Some(event);
            }
        };
        let Some(event) = event else {
            continue;
        };
        if event == DeviceEvent::Power(PowerEvent::Sleeping) {
            application.pause_input();
        }
        // Sleep/wake does not deliberately close the user's board connection.
        // Real link loss is reported by the service; wake never replays a target.
        if matches!(event, DeviceEvent::Power(_)) {
            touch_received = None;
        }
        if event == DeviceEvent::Power(PowerEvent::Awake) {
            // Native wake can repaint any pixels while Xorg is briefly resumed.
            force_full_next = kindle_platform::complete_native_wake(&mut power)?;
        }
        let Some((x, y)) = awake_tap(event, &mut sleeping) else {
            continue;
        };
        touch_received = Some(Instant::now());
        let target = output.hit_test_app(x, y, application.screen());

        if let Some(target) = target {
            eprintln!("kindle-chess: tap ({x},{y}) -> {target:?}");
            if matches!(
                target,
                HitTarget::Refresh | HitTarget::Play(chess_render::PlayTarget::FullRefresh)
            ) {
                force_full_next = true;
                eprintln!("kindle-chess: full-screen refresh requested");
                continue;
            }
            let board_before = application.screen().board().clone();
            let flipped_before = application.screen().flipped();
            if application.tap(target) {
                application.flush();
                eprintln!("kindle-runtime: Exit requested; releasing display and input");
                return Ok(());
            }
            clean_regions = crate::clean_regions_for_board_change(
                &board_before,
                application.screen().board(),
                flipped_before,
                application.screen().flipped(),
                output.layout,
            );
        } else {
            eprintln!("kindle-chess: tap ({x},{y}) -> no target");
            application.flush();
        }
    }
}
pub fn awake_tap(event: DeviceEvent, sleeping: &mut bool) -> Option<(u32, u32)> {
    match event {
        DeviceEvent::Power(PowerEvent::NativeWakeComplete) => None,
        DeviceEvent::Power(event) => {
            *sleeping = event == PowerEvent::Sleeping;
            eprintln!("kindle-chess: power {event:?}");
            None
        }
        DeviceEvent::Tap(_, _) if *sleeping => None,
        DeviceEvent::Tap(x, y) => Some((x, y)),
    }
}

/// A runtime claim below the supervisor's global lock also prevents accidental
/// duplicate writers launched by one supervisor. Abrupt death is reaped by it.
struct FramebufferOwner(std::path::PathBuf);
impl FramebufferOwner {
    fn claim(display_lock: &std::path::Path) -> Result<Self, std::io::Error> {
        let path = display_lock.join("framebuffer");
        std::fs::create_dir(&path)?;
        let owner = Self(path);
        std::fs::write(owner.0.join("pid"), std::process::id().to_string())?;
        Ok(owner)
    }
    fn from_supervisor() -> Result<Self, std::io::Error> {
        let path = std::env::var_os("KINDLE_CHESS_DISPLAY_LOCK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "/tmp/kindle-chess-display.lock".into());
        let owner = std::env::var("KINDLE_CHESS_DISPLAY_OWNER").unwrap_or_default();
        if owner.parse::<u32>().ok().filter(|pid| *pid > 0).is_none()
            || std::fs::read_to_string(path.join("launcher.pid"))
                .ok()
                .as_deref()
                .map(str::trim)
                != Some(owner.as_str())
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "Launch through kindle_launch.sh to acquire exclusive display ownership",
            ));
        }
        Self::claim(&path)
    }
}
impl Drop for FramebufferOwner {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(self.0.join("pid"));
        let _ = std::fs::remove_dir(&self.0);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_one_runtime_can_claim_framebuffer_and_drop_releases_it() {
        let directory =
            std::env::temp_dir().join(format!("kindle-runtime-owner-{}", std::process::id()));
        std::fs::create_dir(&directory).unwrap();
        let first = FramebufferOwner::claim(&directory).unwrap();
        assert!(FramebufferOwner::claim(&directory).is_err());
        drop(first);
        assert!(!directory.join("framebuffer").exists());
        drop(FramebufferOwner::claim(&directory).unwrap());
        std::fs::remove_dir(directory).unwrap();
    }
}
