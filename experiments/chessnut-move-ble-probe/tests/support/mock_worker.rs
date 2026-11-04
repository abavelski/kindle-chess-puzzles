//! Test-only worker; no BLE, native libraries, privileges, or motor access.
use chessnut_ble_probe::ipc::*;
use std::io::{self, BufRead, Write};
fn emit(id: u32, report: Report) {
    println!(
        "{}",
        serde_json::to_string(&Envelope { v: 1, id, report }).unwrap()
    );
    io::stdout().flush().unwrap();
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
            Operation::Discover if scenario.starts_with("database") => {
                let mut records = [
                    (POSITION_SERVICE, POSITION_UUID, 0x10),
                    (COMMAND_SERVICE, WRITE_UUID, 0x0c),
                    (COMMAND_SERVICE, RESPONSE_UUID, 0x10),
                ]
                .map(
                    |(service, uuid, properties)| chessnut_ble_probe::discovery::GattRecord {
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
