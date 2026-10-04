use chess_render::{DisplayMetrics, HitTarget, Layout};
use kindle_platform::{
    is_finger_touchscreen_candidate, AxisRange, DeviceCapabilities, InputCandidate, MtDecoder,
    RawInputEvent, Rotation, TapPolicy, TapRecognizer, TouchEvent, TouchPhase, TouchTransform,
};

const METRICS: DisplayMetrics = DisplayMetrics {
    width: 1860,
    height: 2480,
    dpi: 300,
};

fn bitmap(codes: &[u8]) -> u64 {
    codes
        .iter()
        .fold(0_u64, |bits, code| bits | (1_u64 << code))
}

#[test]
fn device_selection_requires_multitouch_axes_and_rejects_stylus_named_devices() {
    let finger = InputCandidate {
        event_name: "event4".into(),
        name: "pt_mt".into(),
        capabilities: DeviceCapabilities {
            event_types_low64: bitmap(&[3]),
            abs_low64: bitmap(&[0x35, 0x36, 0x39]),
        },
    };
    assert!(is_finger_touchscreen_candidate(&finger));

    let missing_tracking = InputCandidate {
        capabilities: DeviceCapabilities {
            event_types_low64: bitmap(&[3]),
            abs_low64: bitmap(&[0x35, 0x36]),
        },
        ..finger.clone()
    };
    assert!(!is_finger_touchscreen_candidate(&missing_tracking));

    for name in ["WacomDigitizer", "stylus-custom", "active pen"] {
        let stylus = InputCandidate {
            name: name.into(),
            ..finger.clone()
        };
        assert!(!is_finger_touchscreen_candidate(&stylus));
    }
}

#[test]
fn task00_declared_ranges_normalize_directly_to_scribe_pixels() {
    let transform = TouchTransform::new(
        AxisRange::new(0, 1859).unwrap(),
        AxisRange::new(0, 2479).unwrap(),
        METRICS,
        Rotation::Deg0,
    )
    .unwrap();

    assert_eq!(transform.map(0, 0), (0, 0));
    assert_eq!(transform.map(1859, 2479), (1859, 2479));
    assert_eq!(transform.map(930, 1240), (930, 1240));

    for measured in [
        (20, 530),
        (1785, 578),
        (942, 1236),
        (82, 2434),
        (1789, 2446),
    ] {
        assert_eq!(
            transform.map(measured.0, measured.1),
            (
                u32::try_from(measured.0).unwrap(),
                u32::try_from(measured.1).unwrap(),
            )
        );
    }
}

#[test]
fn rotation_transform_maps_corners_for_all_supported_orientations() {
    let raw_x = AxisRange::new(0, 99).unwrap();
    let raw_y = AxisRange::new(0, 199).unwrap();

    let normal = TouchTransform::new(
        raw_x,
        raw_y,
        DisplayMetrics {
            width: 100,
            height: 200,
            dpi: 300,
        },
        Rotation::Deg0,
    )
    .unwrap();
    assert_eq!(normal.map(0, 0), (0, 0));
    assert_eq!(normal.map(99, 199), (99, 199));

    let cw = TouchTransform::new(
        raw_x,
        raw_y,
        DisplayMetrics {
            width: 200,
            height: 100,
            dpi: 300,
        },
        Rotation::Deg90,
    )
    .unwrap();
    assert_eq!(cw.map(0, 0), (199, 0));
    assert_eq!(cw.map(99, 199), (0, 99));

    let upside_down = TouchTransform::new(
        raw_x,
        raw_y,
        DisplayMetrics {
            width: 100,
            height: 200,
            dpi: 300,
        },
        Rotation::Deg180,
    )
    .unwrap();
    assert_eq!(upside_down.map(0, 0), (99, 199));
    assert_eq!(upside_down.map(99, 199), (0, 0));

    let ccw = TouchTransform::new(
        raw_x,
        raw_y,
        DisplayMetrics {
            width: 200,
            height: 100,
            dpi: 300,
        },
        Rotation::Deg270,
    )
    .unwrap();
    assert_eq!(ccw.map(0, 0), (0, 99));
    assert_eq!(ccw.map(99, 199), (199, 0));
}

#[test]
fn multitouch_decoder_emits_down_move_and_up_from_task00_event_shape() {
    let mut decoder = MtDecoder::default();
    let mut emitted = Vec::new();
    for event in [
        RawInputEvent::new(10, 0, 3, 0x39, 42),
        RawInputEvent::new(10, 1_000, 3, 0x35, 942),
        RawInputEvent::new(10, 2_000, 3, 0x36, 1236),
        RawInputEvent::new(10, 3_000, 0, 0, 0),
        RawInputEvent::new(10, 4_000, 3, 0x35, 950),
        RawInputEvent::new(10, 5_000, 3, 0x36, 1240),
        RawInputEvent::new(10, 6_000, 0, 0, 0),
        RawInputEvent::new(10, 7_000, 3, 0x39, -1),
        RawInputEvent::new(10, 8_000, 0, 0, 0),
    ] {
        if let Some(event) = decoder.push(event) {
            emitted.push(event);
        }
    }

    assert_eq!(emitted.len(), 3);
    assert_eq!(emitted[0].phase, TouchPhase::Down);
    assert_eq!((emitted[0].x, emitted[0].y), (942, 1236));
    assert_eq!(emitted[1].phase, TouchPhase::Move);
    assert_eq!((emitted[1].x, emitted[1].y), (950, 1240));
    assert_eq!(emitted[2].phase, TouchPhase::Up);
}

#[test]
fn tap_policy_accepts_small_fast_motion_and_rejects_drag_or_long_press() {
    let policy = TapPolicy {
        max_movement_px: 48,
        max_duration_ms: 750,
    };

    let mut recognizer = TapRecognizer::new(policy);
    assert_eq!(
        recognizer.push(TouchEvent::new(TouchPhase::Down, 100, 100, 1_000)),
        None
    );
    assert_eq!(
        recognizer.push(TouchEvent::new(TouchPhase::Move, 120, 115, 1_200)),
        None
    );
    assert_eq!(
        recognizer.push(TouchEvent::new(TouchPhase::Up, 122, 116, 1_300)),
        Some((122, 116))
    );

    let mut drag = TapRecognizer::new(policy);
    drag.push(TouchEvent::new(TouchPhase::Down, 100, 100, 1_000));
    drag.push(TouchEvent::new(TouchPhase::Move, 200, 100, 1_100));
    assert_eq!(
        drag.push(TouchEvent::new(TouchPhase::Up, 200, 100, 1_200)),
        None
    );

    let mut long = TapRecognizer::new(policy);
    long.push(TouchEvent::new(TouchPhase::Down, 100, 100, 1_000));
    assert_eq!(
        long.push(TouchEvent::new(TouchPhase::Up, 100, 100, 2_000)),
        None
    );
}

#[test]
fn normalized_touch_coordinates_feed_the_same_renderer_hit_targets() {
    let layout = Layout::new(METRICS).unwrap();
    let transform = TouchTransform::new(
        AxisRange::new(0, 1859).unwrap(),
        AxisRange::new(0, 2479).unwrap(),
        METRICS,
        Rotation::Deg0,
    )
    .unwrap();

    for display_square in [0_usize, 7, 27, 36, 56, 63] {
        let rect = layout.square_rect(display_square);
        let raw_x = i32::try_from(rect.x + rect.width / 2).unwrap();
        let raw_y = i32::try_from(rect.y + rect.height / 2).unwrap();
        let (x, y) = transform.map(raw_x, raw_y);
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
fn exclusive_input_failure_has_context_and_cannot_be_silently_shared() {
    let error = kindle_platform::InputError::ExclusiveGrab(std::io::Error::from_raw_os_error(16));
    assert!(error.to_string().starts_with("exclusive finger input: "));
}

fn pen_sequence(events: &[(u16, u16, i32)]) -> Vec<(u32, u32)> {
    let mut decoder = kindle_platform::PenTapDecoder::new(
        kindle_platform::scribe_pen_transform(METRICS).unwrap(),
        TapPolicy::scribe_default(),
    );
    events
        .iter()
        .enumerate()
        .filter_map(|(i, &(kind, code, value))| {
            decoder.push(RawInputEvent::new(1, i as u32 * 1_000, kind, code, value))
        })
        .collect()
}

#[test]
fn pen_contact_noise_emits_one_tap_and_ignores_hover_pressure_tilt() {
    assert_eq!(
        pen_sequence(&[
            (1, 0x140, 1),
            (3, 0, 930),
            (3, 1, 1240),
            (0, 0, 0),
            (1, 0x14a, 1),
            (3, 0x18, 50),
            (0, 0, 0),
            (3, 0, 932),
            (3, 0x1a, 20),
            (0, 0, 0),
            (1, 0x14a, 0),
            (0, 0, 0),
            (1, 0x140, 0),
            (0, 0, 0),
        ]),
        vec![(931, 1240)]
    );
    assert!(pen_sequence(&[
        (1, 0x140, 1),
        (3, 0, 100),
        (3, 1, 200),
        (0, 0, 0),
        (3, 0, 110),
        (0, 0, 0),
        (1, 0x140, 0),
        (0, 0, 0)
    ])
    .is_empty());
}

#[test]
fn pen_repeated_contacts_use_fresh_positions_and_reject_out_of_range_and_drag() {
    assert_eq!(
        pen_sequence(&[
            (3, 0, 0),
            (3, 1, 0),
            (1, 0x14a, 1),
            (0, 0, 0),
            (1, 0x14a, 0),
            (0, 0, 0),
            (3, 0, 1860),
            (3, 1, 2480),
            (1, 0x14a, 1),
            (0, 0, 0),
            (1, 0x14a, 0),
            (0, 0, 0),
        ]),
        vec![(0, 0), (1859, 2479)]
    );
    for bad in [
        vec![
            (3, 0, -1),
            (3, 1, 100),
            (1, 0x14a, 1),
            (0, 0, 0),
            (3, 0, 100),
            (0, 0, 0),
            (1, 0x14a, 0),
            (0, 0, 0),
        ],
        vec![
            (3, 0, 100),
            (3, 1, 100),
            (1, 0x14a, 1),
            (0, 0, 0),
            (3, 0, 200),
            (0, 0, 0),
            (3, 0, 100),
            (1, 0x14a, 0),
            (0, 0, 0),
        ],
    ] {
        assert!(pen_sequence(&bad).is_empty());
    }
}

#[test]
fn pen_discovery_selects_only_the_verified_virtual_stream() {
    let candidate = InputCandidate {
        event_name: "event99".into(),
        name: "stylus-custom".into(),
        capabilities: DeviceCapabilities {
            event_types_low64: bitmap(&[1, 3]),
            abs_low64: bitmap(&[0, 1]),
        },
    };
    assert!(kindle_platform::is_scribe_pen_candidate(&candidate));
    assert!(!kindle_platform::is_scribe_pen_candidate(&InputCandidate {
        name: "WacomDigitizer".into(),
        ..candidate.clone()
    }));
    assert!(!kindle_platform::is_scribe_pen_candidate(&InputCandidate {
        capabilities: DeviceCapabilities {
            event_types_low64: bitmap(&[3]),
            abs_low64: bitmap(&[0, 1])
        },
        ..candidate
    }));
}

#[test]
fn recorded_task00_pen_trace_emits_exactly_five_taps() {
    let mut decoder = kindle_platform::PenTapDecoder::new(
        kindle_platform::scribe_pen_transform(METRICS).unwrap(),
        TapPolicy::scribe_default(),
    );
    let taps = include_bytes!("../../../tests/fixtures/input/scribe-virtual-pen.bin")
        .chunks_exact(16)
        .filter_map(|b| {
            decoder.push(RawInputEvent::new(
                u32::from_le_bytes(b[0..4].try_into().unwrap()),
                u32::from_le_bytes(b[4..8].try_into().unwrap()),
                u16::from_le_bytes(b[8..10].try_into().unwrap()),
                u16::from_le_bytes(b[10..12].try_into().unwrap()),
                i32::from_le_bytes(b[12..16].try_into().unwrap()),
            ))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        taps,
        vec![(64, 86), (1799, 518), (900, 1130), (46, 2431), (1808, 2424)]
    );
}

#[test]
fn pen_and_finger_resolve_the_same_rich_move_chip_targets() {
    use chess_core::{parse_puzzle_file, Action, ActiveCollection, AppState, Progress};
    let mut app = AppState::new(
        ActiveCollection::from_collection(
            "puzzles-test.json",
            parse_puzzle_file(include_bytes!(
                "../../../tests/fixtures/rich-analysis/valid-rich.json"
            ))
            .unwrap(),
        ),
        Progress::new(),
    );
    app.dispatch(Action::OpenAnalysis);
    let output = chess_render::render(&app, METRICS).unwrap();
    let panel = output.analysis.as_ref().unwrap();
    assert!(!panel.move_chips.is_empty());
    for chip in &panel.move_chips {
        let x = chip.rect.x + chip.rect.width / 2;
        let y = chip.rect.y + chip.rect.height / 2;
        let raw_x = ((u64::from(x) * 1860 + 929) / 1859) as i32;
        let raw_y = ((u64::from(y) * 2480 + 1239) / 2479) as i32;
        let taps = pen_sequence(&[
            (3, 0, raw_x),
            (3, 1, raw_y),
            (1, 0x14a, 1),
            (0, 0, 0),
            (1, 0x14a, 0),
            (0, 0, 0),
        ]);
        assert_eq!(taps, vec![(x, y)]);
        assert_eq!(
            output.hit_test_app(x, y, &app),
            output.hit_test_app(taps[0].0, taps[0].1, &app)
        );
        assert_eq!(
            output.hit_test_app(x, y, &app),
            Some(HitTarget::AnalysisMove(chip.node))
        );
    }
}

#[test]
fn pen_eraser_and_long_contact_do_not_activate_targets() {
    assert!(pen_sequence(&[
        (1, 0x141, 1),
        (0, 0, 0),
        (3, 0, 100),
        (3, 1, 100),
        (1, 0x14a, 1),
        (0, 0, 0),
        (1, 0x14a, 0),
        (0, 0, 0)
    ])
    .is_empty());
    let mut decoder = kindle_platform::PenTapDecoder::new(
        kindle_platform::scribe_pen_transform(METRICS).unwrap(),
        TapPolicy::scribe_default(),
    );
    for (kind, code, value) in [(3, 0, 100), (3, 1, 100), (1, 0x14a, 1), (0, 0, 0)] {
        assert!(decoder
            .push(RawInputEvent::new(1, 0, kind, code, value))
            .is_none());
    }
    decoder.push(RawInputEvent::new(2, 0, 1, 0x14a, 0));
    assert!(decoder.push(RawInputEvent::new(2, 0, 0, 0, 0)).is_none());
}
