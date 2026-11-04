#![cfg(feature = "mock-worker")]
use chessnut_ble_probe::ipc::{Client, Error, Operation, Report, WriteMode, MAX_FRAME};
use chessnut_move::protocol::Command as BoardCommand;
use std::{io::Cursor, process::Command, time::Duration};

fn worker(scenario: &str) -> Client {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mock-worker"));
    command.arg(scenario);
    Client::spawn(&mut command, Duration::from_secs(2)).unwrap()
}
fn connect(client: &mut Client) {
    client
        .request(Operation::Connect {
            address: "00:00:00:00:00:00".into(),
        })
        .unwrap();
}

#[test]
fn success_roundtrip_and_shutdown() {
    let mut client = worker("success");
    connect(&mut client);
    assert!(matches!(
        client.request(Operation::Discover).unwrap(),
        Report::Services { .. }
    ));
    client.request(Operation::Subscribe).unwrap();
    let notification = client
        .next_notification(Duration::from_millis(250))
        .unwrap();
    assert!(matches!(
        notification,
        chessnut_move::protocol::BoardEvent::BatteryStatus(_)
    ));
    client
        .request(Operation::from_command(&BoardCommand::read_battery_level()))
        .unwrap();
    client.interrupt().unwrap();
    assert!(client.is_closed());
}

#[test]
fn write_kind_is_explicit_and_all_bytes_survive_ipc() {
    for command in [
        BoardCommand::read_battery_level(),
        BoardCommand::stop_auto_move(),
    ] {
        let operation = Operation::from_command(&command);
        let (bytes, write_kind) = match &operation {
            Operation::Write { bytes, write_kind } => (bytes, *write_kind),
            _ => panic!("expected write"),
        };
        assert_eq!(bytes, command.bytes());
        assert_eq!(
            write_kind,
            if bytes.len() == 35 {
                WriteMode::WithoutResponse
            } else {
                WriteMode::WithResponse
            }
        );
        assert_eq!(
            serde_json::from_str::<Operation>(&serde_json::to_string(&operation).unwrap()).unwrap(),
            operation
        );
    }
}

#[test]
fn wrong_uuid_and_malformed_notification_are_errors() {
    let mut client = worker("wrong-uuid");
    connect(&mut client);
    assert_eq!(client.request(Operation::Discover), Err(Error::WrongUuid));
    for scenario in ["malformed-notification", "unknown-source"] {
        let mut client = worker(scenario);
        connect(&mut client);
        client.request(Operation::Subscribe).unwrap();
        assert_eq!(
            client.next_notification(Duration::from_millis(250)),
            Err(Error::MalformedNotification)
        );
    }
}

#[test]
fn write_failure_timeout_disconnect_and_worker_crash_are_distinct() {
    for (scenario, expected) in [
        ("write-failure", Error::Worker("write_rejected".into())),
        ("timeout", Error::Timeout),
        ("disconnect", Error::Disconnected),
        ("crash", Error::WorkerExited),
        ("bad-id", Error::Protocol),
        ("bad-version", Error::Protocol),
        ("oversized", Error::Protocol),
    ] {
        let mut client = worker(scenario);
        connect(&mut client);
        assert_eq!(
            client.request(Operation::from_command(&BoardCommand::read_battery_level())),
            Err(expected),
            "{scenario}"
        );
        assert!(client.is_closed());
        assert_eq!(client.request(Operation::Discover), Err(Error::Closed));
    }
}

#[test]
fn interrupted_session_sends_disconnect_then_shutdown() {
    // The mock only acknowledges Shutdown after receiving Disconnect.
    let mut client = worker("success");
    connect(&mut client);
    client.interrupt().unwrap();
    assert!(client.is_closed());
}

#[test]
fn real_sigterm_worker_exit_is_reported_and_reaped() {
    let mut client = worker("success");
    connect(&mut client);
    assert!(Command::new("kill")
        .args(["-TERM", &client.worker_id().to_string()])
        .status()
        .unwrap()
        .success());
    assert_eq!(
        client.next_notification(Duration::from_secs(1)),
        Err(Error::WorkerExited)
    );
    assert!(client.is_closed());
}

#[test]
fn framing_is_bounded_versioned_and_rejects_partial_eof() {
    use chessnut_ble_probe::ipc::read_report;
    for data in [
        vec![b'x'; MAX_FRAME + 1],
        b"{}\n".to_vec(),
        b"{\"v\":2,\"id\":1,\"report\":{\"type\":\"ok\"}}\n".to_vec(),
        b"{\"v\":1,\"id\":1,\"report\":{\"type\":\"ok\"}}".to_vec(),
    ] {
        assert_eq!(read_report(&mut Cursor::new(data)), Err(Error::Protocol));
    }
    assert_eq!(
        read_report(&mut Cursor::new(Vec::<u8>::new())),
        Err(Error::WorkerExited)
    );
}
