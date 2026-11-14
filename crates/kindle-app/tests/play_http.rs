use chess_core::play::{Request, Response, MAX_RESPONSE};
use kindle_app::play_http::{Endpoint, HttpClient};
use std::{
    io::{Read, Write},
    net::TcpListener,
    time::Duration,
};
fn server(
    status: u16,
    body: String,
    delay: Duration,
) -> (Endpoint, std::thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = Endpoint::parse(&format!("http://{}", listener.local_addr().unwrap())).unwrap();
    let handle = std::thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut buf = [0; 4096];
        let n = socket.read(&mut buf).unwrap();
        std::thread::sleep(delay);
        let _=write!(socket,"HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
        String::from_utf8_lossy(&buf[..n]).into_owned()
    });
    (endpoint, handle)
}
#[test]
fn validates_base_url_and_exact_routes() {
    for bad in [
        "https://127.0.0.1:80",
        "http://host:80",
        "http://127.0.0.1:0",
        "http://127.0.0.1:80/x",
        "http://user@127.0.0.1:80",
        "http://0.0.0.0:80",
        "http://127.0.0.1:80?x",
    ] {
        assert!(Endpoint::parse(bad).is_err(), "{bad}");
    }
    let (endpoint, handle) = server(
        200,
        r#"{"apiVersion":1,"maiaAvailable":true}"#.into(),
        Duration::ZERO,
    );
    assert!(matches!(
        HttpClient::new(endpoint).execute(&Request::Health),
        Ok(Response::Health(true))
    ));
    let request = handle.join().unwrap();
    assert!(request.starts_with("GET /api/v1/health HTTP/1.1\r\n"));
    for (req, path) in [
        (
            Request::New {
                rating: 1500,
                player_color: chess_core::Color::White,
            },
            "/api/v1/games",
        ),
        (
            Request::Move {
                uci: "e2e4".into(),
                expected_ply: 0,
            },
            "/api/v1/games/current/moves",
        ),
        (Request::Resign, "/api/v1/games/current/resign"),
    ] {
        let (method, route, body) = req.wire();
        assert_eq!(method, "POST");
        assert_eq!(route, path);
        assert!(body.len() < 4096);
    }
}
#[test]
fn refused_timeout_errors_oversize_and_reconnect() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    drop(listener);
    assert!(HttpClient::new(Endpoint::parse(&url).unwrap())
        .execute(&Request::Current)
        .is_err());
    let (endpoint, handle) = server(200, "{}".into(), Duration::from_millis(100));
    let mut client = HttpClient::new(endpoint);
    client.timeout = Duration::from_millis(20);
    assert!(client
        .execute(&Request::Current)
        .unwrap_err()
        .contains("timed out"));
    handle.join().unwrap();
    for (status, code) in [
        (503, "ENGINE_UNAVAILABLE"),
        (504, "ENGINE_TIMEOUT"),
        (409, "STALE_PLY"),
        (422, "ILLEGAL_MOVE"),
    ] {
        let (endpoint, handle) = server(
            status,
            format!(r#"{{"error":{{"code":"{code}","message":"try refresh"}}}}"#),
            Duration::ZERO,
        );
        assert!(
            matches!(HttpClient::new(endpoint).execute(&Request::Current),Ok(Response::Error{code:c,..}) if c==code)
        );
        handle.join().unwrap();
    }
    for body in ["{".into(), " ".repeat(MAX_RESPONSE + 1)] {
        let (endpoint, handle) = server(200, body, Duration::ZERO);
        assert!(HttpClient::new(endpoint)
            .execute(&Request::Current)
            .is_err());
        handle.join().unwrap();
    }
    let (endpoint, handle) = server(
        404,
        r#"{"error":{"code":"NO_GAME","message":"restart"}}"#.into(),
        Duration::ZERO,
    );
    assert!(matches!(
        HttpClient::new(endpoint).execute(&Request::Current),
        Ok(Response::NoGame)
    ));
    handle.join().unwrap();
}

#[test]
fn black_creation_parses_extended_snapshot_and_sends_exact_color() {
    let body=serde_json::json!({"apiVersion":1,"rating":1500,"playerColor":"black","fen":"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1","moves":["e2e4"],"sanMoves":["e4"],"ply":1,"status":"playing","result":null,"lastMove":"e2e4","gameId":"test-game","startedAt":"2026-11-09T12:00:00Z","lastMoveAt":"2026-11-09T12:00:01Z"}).to_string();
    let (endpoint, handle) = server(201, body, Duration::ZERO);
    let response = HttpClient::new(endpoint)
        .execute(&Request::New {
            rating: 1500,
            player_color: chess_core::Color::Black,
        })
        .unwrap();
    let Response::Game(state) = response else {
        panic!("expected authoritative game")
    };
    assert_eq!(state.player_color, "black");
    assert_eq!(state.move_label(1), Some("e4"));
    assert_eq!(state.ply, 1);
    let wire = handle.join().unwrap();
    assert!(wire.starts_with("POST /api/v1/games HTTP/1.1\r\n"));
    let body = wire.split("\r\n\r\n").nth(1).unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(body).unwrap(),
        serde_json::json!({"rating":1500,"playerColor":"black"})
    );
}
