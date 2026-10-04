use kindle_platform::PowerEvent;

#[test]
fn only_screensaver_transitions_change_sleep_overlay() {
    assert_eq!(
        PowerEvent::parse("goingToScreenSaver 2"),
        Some(PowerEvent::Sleeping)
    );
    assert_eq!(
        PowerEvent::parse("[12:13:14.123] goingToScreenSaver 4"),
        Some(PowerEvent::Sleeping)
    );
    assert_eq!(
        PowerEvent::parse("outOfScreenSaver 1"),
        Some(PowerEvent::Awake)
    );
    assert_eq!(
        PowerEvent::parse("outOfScreenSaver 6"),
        Some(PowerEvent::Awake)
    );
    for line in [
        "readyToSuspend 10",
        "wakeupFromSuspend 0",
        "t1TimerReset",
        "unexpectedgoingToScreenSaver",
        "",
    ] {
        assert_eq!(PowerEvent::parse(line), None);
    }
}
