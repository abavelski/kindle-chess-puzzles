use chessnut_ble_probe::preflight::run_worker;
use std::{process::Command, sync::atomic::AtomicBool, time::Duration};

#[test]
fn preflight_propagates_worker_failure_and_accepts_success() {
    let cancelled = AtomicBool::new(false);
    assert!(run_worker(
        &mut Command::new("true"),
        Duration::from_secs(1),
        &cancelled
    )
    .is_ok());
    assert!(run_worker(
        &mut Command::new("false"),
        Duration::from_secs(1),
        &cancelled
    )
    .unwrap_err()
    .contains("failed"));
}
#[test]
fn preflight_kills_and_reaps_worker_on_timeout_or_interruption() {
    for (flag, timeout, expected) in [
        (false, Duration::from_millis(50), "timed out"),
        (true, Duration::from_secs(2), "interrupted"),
    ] {
        let mut command = Command::new("sleep");
        command.arg("10");
        assert!(run_worker(&mut command, timeout, &AtomicBool::new(flag))
            .unwrap_err()
            .contains(expected));
    }
}

#[test]
fn controller_sigint_and_sigterm_reap_the_preflight_worker() {
    use std::{fs, os::unix::fs::PermissionsExt, process::Stdio, time::Instant};
    for signal in ["-INT", "-TERM"] {
        let directory = std::env::temp_dir().join(format!(
            "chessnut-preflight-{}-{signal}",
            std::process::id()
        ));
        fs::create_dir_all(&directory).unwrap();
        let controller = directory.join("chessnut-ble-probe");
        fs::copy(env!("CARGO_BIN_EXE_chessnut-ble-probe"), &controller).unwrap();
        let worker = directory.join("kindlebt-worker");
        fs::write(&worker, "#!/bin/sh\necho $$ > worker.pid\nexec sleep 30\n").unwrap();
        fs::set_permissions(&worker, fs::Permissions::from_mode(0o755)).unwrap();
        let mut child = Command::new(controller)
            .arg("preflight")
            .current_dir(&directory)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        while !directory.join("worker.pid").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(10));
        }
        let worker_pid = fs::read_to_string(directory.join("worker.pid")).unwrap();
        assert!(Command::new("kill")
            .args([signal, &child.id().to_string()])
            .status()
            .unwrap()
            .success());
        let deadline = Instant::now() + Duration::from_secs(3);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("controller did not exit after signal");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        assert!(!status.success());
        assert!(!Command::new("kill")
            .args(["-0", worker_pid.trim()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success());
        fs::remove_dir_all(directory).unwrap();
    }
}
