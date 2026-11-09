use chess_core::play::{Request, Response, Snapshot};
use chess_render::{HitTarget, PlayTarget};
use kindle_app::{
    play_http::PlayClient,
    products::PlayRuntime,
    runtime::{Application, Poll},
};
use std::{
    sync::mpsc,
    time::{Duration, Instant},
};
struct Fake {
    entered: mpsc::Sender<Request>,
    replies: std::sync::Mutex<mpsc::Receiver<Result<Response, String>>>,
}
impl PlayClient for Fake {
    fn execute(&self, r: &Request) -> Result<Response, String> {
        self.entered.send(r.clone()).unwrap();
        self.replies.lock().unwrap().recv().unwrap()
    }
}
fn snapshot() -> Snapshot {
    Snapshot::parse(&serde_json::json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":chess_core::play::START_FEN,"moves":[],"ply":0,"status":"playing","result":null,"lastMove":null})).unwrap()
}
fn poll_changed(runtime: &mut PlayRuntime) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if runtime.poll() == Poll::Changed {
            return;
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(1));
    }
}
#[test]
fn worker_is_nonblocking_sleep_safe_and_never_replays_post() {
    let (entered, requests) = mpsc::channel();
    let (reply, replies) = mpsc::channel();
    let mut runtime = PlayRuntime::with_client(Fake {
        entered,
        replies: std::sync::Mutex::new(replies),
    });
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Health
    );
    assert_eq!(runtime.poll(), Poll::Idle);
    assert!(!runtime.tap(HitTarget::Play(PlayTarget::NewGame)));
    reply.send(Ok(Response::Health(true))).unwrap();
    poll_changed(&mut runtime);
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Current
    );
    reply.send(Ok(Response::Game(snapshot()))).unwrap();
    poll_changed(&mut runtime);
    let before = runtime.state().board.clone();
    runtime.tap(HitTarget::Play(PlayTarget::Square(52)));
    runtime.tap(HitTarget::Play(PlayTarget::Square(36)));
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Move {
            uci: "e2e4".into(),
            expected_ply: 0
        }
    );
    assert_eq!(runtime.state().board, before);
    runtime.pause_input();
    runtime.tap(HitTarget::Play(PlayTarget::Square(51)));
    assert!(requests.try_recv().is_err());
    reply.send(Err("Timeout".into())).unwrap();
    poll_changed(&mut runtime);
    assert_eq!(runtime.state().board, before);
    runtime.tap(HitTarget::Play(PlayTarget::Retry));
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Health
    );
    reply.send(Ok(Response::Health(true))).unwrap();
    poll_changed(&mut runtime);
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Current
    );
    let updated=Snapshot::parse(&serde_json::json!({"apiVersion":1,"rating":1500,"playerColor":"white","fen":"rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq e6 0 2","moves":["e2e4","e7e5"],"ply":2,"status":"playing","result":null,"lastMove":"e7e5"})).unwrap();
    runtime.pause_input();
    reply.send(Ok(Response::Game(updated))).unwrap();
    poll_changed(&mut runtime);
    assert_eq!(runtime.state().snapshot.as_ref().unwrap().ply, 2);
    runtime.tap(HitTarget::Play(PlayTarget::Resign));
    assert_eq!(
        requests.recv_timeout(Duration::from_secs(1)).unwrap(),
        Request::Resign
    );
    let start = Instant::now();
    assert!(runtime.tap(HitTarget::Play(PlayTarget::Close)));
    drop(runtime);
    assert!(start.elapsed() < Duration::from_millis(100));
    reply.send(Err("closed".into())).unwrap();
}
#[test]
fn missing_corrupt_and_future_config_bytes_are_preserved_and_retry_reloads() {
    let root = std::env::temp_dir().join(format!("play-config-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let runtime = PlayRuntime::open(root.clone());
    assert!(runtime.state().message.contains("Configure serverUrl"));
    drop(runtime);
    for bytes in [
        b"{".as_slice(),
        br#"{"serverUrl":"http://127.0.0.1:8765","schemaVersion":99}"#,
    ] {
        std::fs::write(root.join("config.json"), bytes).unwrap();
        let mut runtime = PlayRuntime::open(root.clone());
        runtime.tap(HitTarget::Play(PlayTarget::Retry));
        runtime.flush();
        drop(runtime);
        assert_eq!(std::fs::read(root.join("config.json")).unwrap(), bytes);
    }
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn configured_http_worker_recovers_current_state_after_sleep() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
    };
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let root = std::env::temp_dir().join(format!("play-http-runtime-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let mut runtime = PlayRuntime::open(root.clone());
    std::fs::write(
        root.join("config.json"),
        format!(
            r#"{{"serverUrl":"http://{}"}}"#,
            listener.local_addr().unwrap()
        ),
    )
    .unwrap();
    let (arrived, arrival) = mpsc::channel();
    let (release, ready) = mpsc::channel();
    let server = std::thread::spawn(move || {
        for (route, body) in [
            (
                "/api/v1/health",
                serde_json::json!({"apiVersion":1,"maiaAvailable":true}).to_string(),
            ),
            (
                "/api/v1/games/current",
                serde_json::to_string(&snapshot()).unwrap(),
            ),
        ] {
            let (mut socket, _) = listener.accept().unwrap();
            let mut data = Vec::new();
            while !data.windows(4).any(|w| w == b"\r\n\r\n") {
                let mut buf = [0; 1024];
                let n = socket.read(&mut buf).unwrap();
                assert!(n > 0);
                data.extend_from_slice(&buf[..n]);
            }
            assert!(String::from_utf8_lossy(&data).starts_with(&format!("GET {route} HTTP/1.1")));
            if route.ends_with("current") {
                arrived.send(()).unwrap();
                ready.recv().unwrap();
            }
            let response=format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
            socket.write_all(response.as_bytes()).unwrap();
        }
    });
    runtime.tap(HitTarget::Play(PlayTarget::Retry));
    poll_changed(&mut runtime);
    arrival.recv_timeout(Duration::from_secs(2)).unwrap();
    runtime.pause_input();
    assert_eq!(runtime.poll(), Poll::Idle);
    assert!(runtime.state().in_flight());
    release.send(()).unwrap();
    poll_changed(&mut runtime);
    assert!(runtime.state().can_move());
    server.join().unwrap();
    drop(runtime);
    assert_eq!(std::fs::read_dir(&root).unwrap().count(), 1);
    std::fs::remove_dir_all(root).unwrap();
}
