use std::process::Command;
#[test]
fn offline_packet_fen_prints_full_fen_bytes_kind_and_no_write() {
    let fen = "6kq/8/8/8/8/8/8/8 b - - 17 42";
    let output = Command::new(env!("CARGO_BIN_EXE_chessnut-ble-probe"))
        .args(["packet-fen", fen])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains(fen));
    assert!(text.contains("35 bytes; WithoutResponse; Normal; OFFLINE ONLY"));
    assert!(text.contains("42 21 21 00"));
    assert!(text.trim_end().ends_with("01"));
}
#[test]
fn malformed_fen_and_live_commands_fail_explicitly() {
    for args in [
        vec!["packet-fen", "bad"],
        vec!["preflight"],
        vec!["session"],
        vec!["set-fen"],
        vec!["motion-enable"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_chessnut-ble-probe"))
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success());
        if args[0] != "packet-fen" {
            assert!(String::from_utf8_lossy(&output.stderr).contains("UNSUPPORTED"));
        }
    }
}
