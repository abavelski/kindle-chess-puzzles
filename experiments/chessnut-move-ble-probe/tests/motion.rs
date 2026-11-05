use chessnut_ble_probe::{motion::Plan, TargetFen};
fn position(p: &str) -> chessnut_move::protocol::Position {
    TargetFen::parse(&format!("{p} w - - 0 1"))
        .unwrap()
        .position
}
#[test]
fn only_one_reversible_pawn_change_with_clear_path_is_permitted() {
    let source = position("4k3/8/8/8/8/8/4P3/4K3");
    let target = position("4k3/8/8/8/4P3/8/8/4K3");
    let plan = Plan::new(source, target).unwrap();
    assert_eq!(plan.command().bytes().len(), 35);
    assert_eq!(plan.command().bytes()[34], 1);
    assert!(Plan::new(target, source).is_ok());
    for bad in [
        "4k3/8/8/8/8/4P3/4P3/4K3",
        "4k3/8/8/8/8/8/3P4/4K3",
        "4k3/8/8/4P3/8/8/8/4K3",
        "4k3/8/8/8/8/8/4Q3/4K3",
        "4k3/8/8/8/8/8/4P3/3K4",
    ] {
        assert!(Plan::new(source, position(bad)).is_err(), "{bad}");
    }
    assert!(Plan::new(source, source).is_err());
    assert!(Plan::new(
        position("4k3/8/8/8/8/4n3/4P3/4K3"),
        position("4k3/8/8/8/4P3/4n3/8/4K3")
    )
    .is_err());
    assert!(Plan::new(
        position("8/8/8/8/8/8/4P3/4K3"),
        position("8/8/8/8/4P3/8/8/4K3")
    )
    .is_err());
}
#[test]
fn confirmation_is_expiring_source_bound_and_consumed_once() {
    use chessnut_ble_probe::motion::Guard;
    use std::time::{Duration, Instant};
    let s = position("4k3/8/8/8/8/8/4P3/4K3");
    let t = position("4k3/8/8/8/4P3/8/8/4K3");
    let plan = Plan::new(s, t).unwrap();
    let now = Instant::now();
    let mut g = Guard::default();
    assert!(g.preview(plan, now).is_err());
    g.enable();
    g.preview(plan, now).unwrap();
    assert!(g.confirm(s, now).is_ok());
    assert!(g.confirm(s, now).is_err());
    g.enable();
    g.preview(plan, now).unwrap();
    assert!(g.confirm(t, now).is_err());
    g.enable();
    g.preview(plan, now).unwrap();
    assert!(g.confirm(s, now + Duration::from_secs(31)).is_err());
    g.enable();
    g.preview(plan, now).unwrap();
    g.reset();
    assert!(g.confirm(s, now).is_err());
}
#[test]
fn explicit_e2_e7_stop_probe_requires_a_clear_file_and_is_reversible() {
    let source = position("4k3/8/8/8/8/8/4P3/4K3");
    let target = position("4k3/4P3/8/8/8/8/8/4K3");
    assert!(Plan::new(source, target).is_ok());
    assert!(Plan::new(target, source).is_ok());
    let blocked = position("4k3/8/8/8/4n3/8/4P3/4K3");
    let blocked_target = position("4k3/4P3/8/8/4n3/8/8/4K3");
    assert!(Plan::new(blocked, blocked_target).is_err());
    // Longer moves elsewhere remain outside this explicit STOP-test exception.
    assert!(Plan::new(
        position("4k3/8/8/8/8/8/3P4/4K3"),
        position("4k3/3P4/8/8/8/8/8/4K3")
    )
    .is_err());
}
