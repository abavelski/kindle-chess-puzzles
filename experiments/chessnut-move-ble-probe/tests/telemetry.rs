use chessnut_ble_probe::{
    ipc::{POSITION_UUID, RESPONSE_UUID},
    telemetry::{decode_observed, led_command, m3_command_allowed, placement},
};
use chessnut_move::protocol::{Color, Command, Piece, PieceKind, Position, Square};
#[test]
fn only_sdk_non_motor_packets_and_one_square_leds_are_allowed() {
    for command in [
        Command::enable_realtime_updates(),
        Command::read_battery_level(),
        Command::read_piece_status(),
        led_command("e4", "green").unwrap(),
        led_command("clear", "off").unwrap(),
    ] {
        assert!(m3_command_allowed(&command));
    }
    assert!(!m3_command_allowed(&Command::stop_auto_move()));
    let command = led_command("e4", "green").unwrap();
    assert_eq!(command.bytes().len(), 34);
    assert_eq!(
        command.write_kind(),
        chessnut_move::protocol::WriteKind::WithoutResponse
    );
    assert_eq!(&command.bytes()[..2], &[0x43, 0x20]);
    assert!(led_command("e9", "green").is_err());
    assert!(led_command("e4", "purple").is_err());
}
#[test]
fn sensor_placement_does_not_invent_fen_metadata() {
    let mut position = Position::new([None; 64]);
    position.set_piece(
        "e4".parse::<Square>().unwrap(),
        Some(Piece {
            color: Color::White,
            kind: PieceKind::Pawn,
        }),
    );
    assert_eq!(placement(&position), "8/8/8/8/4P3/8/8/8");
}
#[test]
fn bounded_notifications_keep_source_length_and_sdk_event() {
    let observed = decode_observed(RESPONSE_UUID, &[0x41, 3, 0x0c, 1, 72]).unwrap();
    assert_eq!(observed.bytes.len(), 5);
    assert!(
        matches!(observed.event,chessnut_move::protocol::BoardEvent::BatteryStatus(s) if s.percentage==72)
    );
    assert!(decode_observed(POSITION_UUID, &[0; 33]).is_err());
    assert!(decode_observed(RESPONSE_UUID, &[0; 513]).is_err());
    assert!(decode_observed("other", &[0; 34]).is_err());
    let position = decode_observed(POSITION_UUID, &[0; 34]).unwrap();
    assert_eq!(position.bytes.len(), 34);
}
