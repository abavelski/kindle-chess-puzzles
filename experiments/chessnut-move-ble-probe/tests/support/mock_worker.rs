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
    let scenario = std::env::args().nth(1).unwrap();
    let mut disconnected = false;
    for line in io::stdin().lock().lines() {
        let request: Request = serde_json::from_str(&line.unwrap()).unwrap();
        assert_eq!(request.v, 1);
        let id = request.id;
        match request.operation {
            Operation::Connect { .. } => emit(id, Report::Connected),
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
                return;
            }
        }
    }
}
