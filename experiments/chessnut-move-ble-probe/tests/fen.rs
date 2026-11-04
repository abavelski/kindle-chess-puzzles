use chessnut_ble_probe::TargetFen;
use chessnut_move::protocol::{Color, Command, Piece, PieceKind, Square, WriteKind};

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[test]
fn fen_retains_metadata_and_maps_all_squares_and_colors() {
    let target = TargetFen::parse(START).unwrap();
    assert_eq!(target.fen, START);
    for (square, kind, color) in [
        ("a1", PieceKind::Rook, Color::White),
        ("b1", PieceKind::Knight, Color::White),
        ("c1", PieceKind::Bishop, Color::White),
        ("d1", PieceKind::Queen, Color::White),
        ("e1", PieceKind::King, Color::White),
        ("h2", PieceKind::Pawn, Color::White),
        ("a8", PieceKind::Rook, Color::Black),
        ("b8", PieceKind::Knight, Color::Black),
        ("c8", PieceKind::Bishop, Color::Black),
        ("d8", PieceKind::Queen, Color::Black),
        ("e8", PieceKind::King, Color::Black),
        ("h7", PieceKind::Pawn, Color::Black),
    ] {
        assert_eq!(
            target.position.piece_at(square.parse::<Square>().unwrap()),
            Some(Piece { kind, color })
        );
    }
    assert_eq!(
        (0..64)
            .filter(|i| target
                .position
                .piece_at(Square::from_index(*i).unwrap())
                .is_some())
            .count(),
        32
    );
    for rank in 2..6 {
        assert!((rank * 8..rank * 8 + 8).all(|i| target
            .position
            .piece_at(Square::from_index(i).unwrap())
            .is_none()));
    }
}

#[test]
fn rejects_invalid_geometry_characters_and_metadata() {
    for placement in [
        "8/8",
        "8/8/8/8/8/8/8/8/8",
        "7/8/8/8/8/8/8/8",
        "9/8/8/8/8/8/8/8",
        "0/8/8/8/8/8/8/8",
        "7X/8/8/8/8/8/8/8",
        "7♟/8/8/8/8/8/8/8",
        "44/8/8/8/8/8/8/8",
    ] {
        assert!(
            TargetFen::parse(&format!("{placement} w - - 0 1")).is_err(),
            "{placement}"
        );
    }
    for metadata in [
        "",
        "w",
        "x - - 0 1",
        "w KK - 0 1",
        "w x - 0 1",
        "w - e4 0 1",
        "w - - -1 1",
        "w - - 0 0",
        "w - - 0 1 extra",
    ] {
        assert!(
            TargetFen::parse(&format!("8/8/8/8/8/8/8/8 {metadata}")).is_err(),
            "{metadata}"
        );
    }
}

#[test]
fn upstream_h8_g8_fixture_has_exact_35_byte_normal_packet() {
    // Upstream command.rs::auto_move_command_encodes_target_position_and_mode.
    let target = TargetFen::parse("6kq/8/8/8/8/8/8/8 b - - 17 42").unwrap();
    let mut expected = [0; 35];
    expected[..3].copy_from_slice(&[0x42, 0x21, 0x21]);
    expected[34] = 1;
    assert_eq!(target.packet().bytes(), expected);
    assert_eq!(target.packet().write_kind(), WriteKind::WithoutResponse);
}

#[test]
fn full_start_packet_matches_piece_codes_and_reversed_wire_order() {
    let packet = TargetFen::parse(START).unwrap().packet();
    let mut expected = vec![0x42, 0x21, 0x58, 0x23, 0x31, 0x85];
    expected.extend([0x44; 4]);
    expected.extend([0; 16]);
    expected.extend([0x77; 4]);
    expected.extend([0xa6, 0xc9, 0x9b, 0x6a, 1]);
    assert_eq!(packet.bytes(), expected);
    let empty = TargetFen::parse("8/8/8/8/8/8/8/8 w - - 0 1")
        .unwrap()
        .packet();
    assert!(empty.bytes()[2..34].iter().all(|b| *b == 0));
    assert_eq!(empty.bytes()[34], 1);
}

#[test]
fn metadata_does_not_change_placement_packet_and_sdk_write_kinds_differ() {
    let a = TargetFen::parse(START).unwrap();
    let b = TargetFen::parse(&START.replace("w KQkq - 0 1", "b - e3 12 99")).unwrap();
    assert_eq!(a.packet(), b.packet());
    for (command, bytes) in [
        (Command::enable_realtime_updates(), [0x21, 1, 0]),
        (Command::read_battery_level(), [0x41, 1, 0x0c]),
        (Command::read_piece_status(), [0x41, 1, 0x0b]),
    ] {
        assert_eq!(command.bytes(), bytes);
        assert_eq!(command.write_kind(), WriteKind::WithResponse);
    }
    assert_eq!(Command::stop_auto_move().bytes().len(), 35);
    assert_eq!(
        Command::stop_auto_move().write_kind(),
        WriteKind::WithoutResponse
    );
}
