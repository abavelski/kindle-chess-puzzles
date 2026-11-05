//! Test-only worker; no BLE, native libraries, privileges, or motor access.
use chessnut_board_support::ipc::*;
use std::io::{self, BufRead, Write};
fn emit(id: u32, report: Report) {
    println!(
        "{}",
        serde_json::to_string(&Envelope { v: 1, id, report }).unwrap()
    );
    io::stdout().flush().unwrap();
}
fn moved_notification(packed: &[u8]) -> Vec<u8> {
    let mut bytes = vec![1, 0x24];
    bytes.extend_from_slice(packed);
    bytes.extend([0; 4]);
    bytes
}
fn main() {
    let scenario =
        std::env::var("PROBE_MOCK_SCENARIO").unwrap_or_else(|_| std::env::args().nth(1).unwrap());
    if let Ok(path) = std::env::var("PROBE_MOCK_PID_FILE") {
        std::fs::write(path, std::process::id().to_string()).unwrap();
    }
    if let Ok(path) = std::env::var("PROBE_MOCK_CLEANUP_FILE") {
        let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTERM, std::sync::Arc::clone(&flag))
            .unwrap();
        std::thread::spawn(move || loop {
            if flag.load(std::sync::atomic::Ordering::Relaxed) {
                std::fs::write(&path, "TERM cleanup").unwrap();
                std::process::exit(0);
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        });
    }
    let mut disconnected = false;
    let mut motor_sent = false;
    let mut motor_target: Option<Vec<u8>> = None;
    let mut stopped = false;
    let mut armed = false;
    for line in io::stdin().lock().lines() {
        let request: Request = serde_json::from_str(&line.unwrap()).unwrap();
        assert_eq!(request.v, 1);
        let id = request.id;
        match request.operation {
            Operation::Connect { .. } => match scenario.as_str() {
                "connect-timeout" => std::thread::sleep(std::time::Duration::from_secs(10)),
                "connect-crash" => std::process::exit(19),
                "startup-error" => emit(
                    0,
                    Report::Error {
                        code: "required_state_symbol_status_-1".into(),
                    },
                ),
                "connect-error" => emit(
                    id,
                    Report::Error {
                        code: "connect_address_stale_or_unavailable_status_1".into(),
                    },
                ),
                _ => emit(id, Report::Connected),
            },
            Operation::Discover
                if (scenario.starts_with("database")
                    || scenario.starts_with("m3")
                    || scenario.starts_with("m4")) =>
            {
                let mut records = [
                    (POSITION_SERVICE, POSITION_UUID, 0x10),
                    (COMMAND_SERVICE, WRITE_UUID, 0x0c),
                    (COMMAND_SERVICE, RESPONSE_UUID, 0x10),
                ]
                .map(
                    |(service, uuid, properties)| chessnut_board_support::discovery::GattRecord {
                        service: service.into(),
                        uuid: uuid.into(),
                        properties,
                        handle: 1,
                    },
                )
                .to_vec();
                if scenario == "database-wrong" {
                    records[0].uuid = WRITE_UUID.into();
                }
                emit(id, Report::Database { records });
                if let Ok(path) = std::env::var("PROBE_MOCK_READY_FILE") {
                    std::fs::write(path, "ready").unwrap();
                }
                if scenario == "database-disconnect" {
                    emit(0, Report::Disconnected);
                }
            }
            Operation::Discover => emit(
                id,
                Report::Services {
                    uuids: if scenario == "wrong-uuid" {
                        vec!["wrong".into()]
                    } else {
                        [
                            POSITION_SERVICE,
                            POSITION_UUID,
                            COMMAND_SERVICE,
                            WRITE_UUID,
                            RESPONSE_UUID,
                        ]
                        .map(str::to_owned)
                        .to_vec()
                    },
                },
            ),
            Operation::Arm => {
                assert!(scenario.starts_with("m4"));
                assert!(!armed);
                armed = true;
                emit(id, Report::Ok);
            }
            Operation::Capabilities => emit(
                id,
                Report::Capabilities {
                    mtu: if scenario == "m3-mtu-unknown" {
                        None
                    } else if scenario == "m3-mtu-small" {
                        Some(23)
                    } else {
                        Some(64)
                    },
                    callback_status: if scenario == "m3-mtu-unknown" {
                        None
                    } else {
                        Some(0)
                    },
                },
            ),
            Operation::Subscribe if scenario.starts_with("m3") || scenario.starts_with("m4") => {
                emit(id, Report::Ok);
                if scenario == "m4-stream" {
                    let target =
                        chessnut_board_support::TargetFen::parse("4k3/8/8/8/8/8/4P3/4K3 w - - 0 1")
                            .unwrap();
                    let mut bytes = vec![1, 0x24];
                    bytes.extend_from_slice(&target.packet().bytes()[2..34]);
                    std::thread::spawn(move || {
                        for _ in 0..30 {
                            std::thread::sleep(std::time::Duration::from_millis(10));
                            emit(
                                0,
                                Report::Notification {
                                    source: POSITION_UUID.into(),
                                    bytes: bytes.clone(),
                                },
                            );
                        }
                    });
                }
            }
            Operation::Subscribe => {
                emit(id, Report::Ok);
                emit(
                    0,
                    Report::Notification {
                        source: if scenario == "unknown-source" {
                            WRITE_UUID
                        } else {
                            RESPONSE_UUID
                        }
                        .into(),
                        bytes: if scenario == "malformed-notification" {
                            vec![0x41]
                        } else {
                            vec![0x41, 3, 0x0c, 1, 72]
                        },
                    },
                );
            }
            Operation::Write { bytes, write_kind } if scenario.starts_with("m4") => {
                use chessnut_board_support::TargetFen;
                let notification = |placement: &str| {
                    let fen = TargetFen::parse(&format!("{placement} w - - 0 1")).unwrap();
                    let mut bytes = vec![1, 0x24];
                    bytes.extend_from_slice(&fen.packet().bytes()[2..34]);
                    bytes.extend([0; 4]);
                    bytes
                };
                if bytes == [0x21, 1, 0] {
                    emit(id, Report::Ok);
                    emit(
                        0,
                        Report::Notification {
                            source: POSITION_UUID.into(),
                            bytes: if scenario.starts_with("m4-paused") && motor_sent && stopped {
                                moved_notification(motor_target.as_ref().unwrap())
                            } else {
                                notification(if scenario == "m4-wrong-source" {
                                    "4k3/8/8/8/8/4P3/8/4K3"
                                } else if scenario == "m4-return" {
                                    "4k3/4P3/8/8/8/8/8/4K3"
                                } else {
                                    "4k3/8/8/8/8/8/4P3/4K3"
                                })
                            },
                        },
                    );
                    if scenario.starts_with("m4-paused") && motor_sent && stopped {
                        for _ in 0..2 {
                            emit(
                                0,
                                Report::Notification {
                                    source: POSITION_UUID.into(),
                                    bytes: moved_notification(motor_target.as_ref().unwrap()),
                                },
                            );
                        }
                    }
                } else if bytes.len() == 35 && bytes[..2] == [0x42, 0x21] {
                    assert_eq!(write_kind, WriteMode::WithoutResponse);
                    if bytes[2..].iter().all(|b| *b == 0) {
                        armed = false;
                        stopped = true;
                        emit(id, Report::Ok);
                    } else {
                        assert!(armed && !motor_sent);
                        assert_eq!(bytes[34], 1);
                        armed = false;
                        motor_sent = true;
                        motor_target = Some(bytes[2..34].to_vec());
                        emit(id, Report::Ok);
                        if scenario == "m4" || scenario == "m4-return" {
                            let bytes = moved_notification(motor_target.as_ref().unwrap());
                            std::thread::spawn(move || {
                                for _ in 0..5 {
                                    std::thread::sleep(std::time::Duration::from_millis(15));
                                    emit(
                                        0,
                                        Report::Notification {
                                            source: POSITION_UUID.into(),
                                            bytes: bytes.clone(),
                                        },
                                    );
                                }
                            });
                        }
                    }
                } else {
                    panic!("unexpected M4 packet {bytes:?}");
                }
            }
            Operation::Write { bytes, write_kind } if scenario.starts_with("m3") => {
                if scenario == "m3-position-only" {
                    assert_eq!(
                        bytes,
                        [0x21, 1, 0],
                        "position probe must only enable realtime"
                    );
                }
                let allowed = (write_kind == WriteMode::WithResponse
                    && [vec![0x21, 1, 0], vec![0x41, 1, 0x0c], vec![0x41, 1, 0x0b]]
                        .contains(&bytes))
                    || (write_kind == WriteMode::WithoutResponse
                        && bytes.len() == 34
                        && bytes[..2] == [0x43, 0x20]);
                assert!(allowed, "M3 must never dispatch a motor/unknown packet");
                if scenario == "m3-write-failed" {
                    emit(
                        id,
                        Report::Error {
                            code: "write_callback_failed".into(),
                        },
                    );
                    continue;
                }
                if scenario == "m3-disconnect" {
                    emit(0, Report::Disconnected);
                    continue;
                }
                // Synthetic unknown response, not a claim about uncaptured device bytes.
                if bytes == [0x21, 1, 0] && scenario.starts_with("m3-raw") {
                    emit(
                        0,
                        Report::Notification {
                            source: RESPONSE_UUID.into(),
                            bytes: vec![0x21, 1, 0],
                        },
                    );
                }
                emit(id, Report::Ok);
                if bytes == [0x21, 1, 0]
                    && !matches!(scenario.as_str(), "m3-no-position" | "m3-raw-only")
                {
                    emit(
                        0,
                        Report::Notification {
                            source: POSITION_UUID.into(),
                            bytes: if scenario == "m3-malformed" {
                                vec![0; 20]
                            } else {
                                vec![0; 34]
                            },
                        },
                    );
                } else if bytes == [0x41, 1, 0x0c] && scenario != "m3-battery-timeout" {
                    emit(
                        0,
                        Report::Notification {
                            source: RESPONSE_UUID.into(),
                            bytes: vec![0x41, 3, 0x0c, 1, 72],
                        },
                    );
                } else if bytes == [0x41, 1, 0x0b] {
                    let identities = [
                        1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 0x07, 0x07, 0x07, 0x07,
                        0x07, 0x07, 0x07, 0x07, 0x08, 0x08, 0x09, 0x09, 0x0a, 0x0a, 0x0b, 0x0b,
                        0x0c,
                    ];
                    let mut bytes = vec![0x41, 0x89, 0x0b];
                    for identity in identities {
                        bytes.extend([identity, 0, 0, 50]);
                    }
                    emit(
                        0,
                        Report::Notification {
                            source: RESPONSE_UUID.into(),
                            bytes,
                        },
                    );
                }
            }
            Operation::Write { .. } => match scenario.as_str() {
                "write-failure" => emit(
                    id,
                    Report::Error {
                        code: "write_rejected".into(),
                    },
                ),
                "timeout" => std::thread::sleep(std::time::Duration::from_secs(10)),
                "disconnect" => emit(0, Report::Disconnected),
                "crash" => std::process::exit(17),
                "bad-id" => emit(id + 1, Report::Ok),
                "bad-version" => {
                    println!("{{\"v\":2,\"id\":{id},\"report\":{{\"type\":\"ok\"}}}}");
                    io::stdout().flush().unwrap();
                }
                "oversized" => {
                    println!("{}", "x".repeat(MAX_FRAME + 1));
                    io::stdout().flush().unwrap();
                }
                _ => emit(id, Report::Ok),
            },
            Operation::Disconnect => {
                disconnected = true;
                emit(id, Report::Disconnected);
            }
            Operation::Shutdown => {
                assert!(disconnected);
                emit(id, Report::Ok);
                if scenario == "database-shutdown-hang" {
                    // Ignore TERM in this deterministic forced-cleanup case.
                    signal_hook::flag::register(
                        signal_hook::consts::SIGTERM,
                        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
                    )
                    .unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(10));
                }
                return;
            }
        }
    }
    if scenario == "database-signal" {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
    }
}
