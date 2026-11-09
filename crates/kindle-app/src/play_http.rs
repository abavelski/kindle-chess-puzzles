//! Bounded plain HTTP transport for Play's fixed trusted-LAN routes.
use chess_core::play::{parse_response, Request, Response, MAX_RESPONSE};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, SocketAddrV4, TcpStream},
    time::{Duration, Instant},
};
#[derive(Clone, Debug)]
pub struct Endpoint {
    address: SocketAddrV4,
}
impl Endpoint {
    pub fn parse(url: &str) -> Result<Self, String> {
        if url.len() > 128 {
            return Err("serverUrl too long".into());
        }
        let authority = url
            .strip_prefix("http://")
            .ok_or("serverUrl must use http and an IPv4 address")?;
        let (host, port) = authority.split_once(':').unwrap_or((authority, "80"));
        let ip: Ipv4Addr = host
            .parse()
            .map_err(|_| "serverUrl must use a desktop LAN IPv4 address")?;
        let port: u16 = port.parse().map_err(|_| "Invalid serverUrl port")?;
        if port == 0 || !(ip.is_private() || ip.is_loopback() || ip.is_link_local()) {
            return Err("serverUrl must use a trusted LAN IPv4 address and nonzero port".into());
        }
        Ok(Self {
            address: SocketAddrV4::new(ip, port),
        })
    }
}
pub trait PlayClient: Send + 'static {
    fn execute(&self, request: &Request) -> Result<Response, String>;
}
pub struct HttpClient {
    endpoint: Endpoint,
    pub timeout: Duration,
}
impl HttpClient {
    pub fn new(endpoint: Endpoint) -> Self {
        Self {
            endpoint,
            timeout: Duration::from_secs(195),
        }
    }
    pub fn execute(&self, request: &Request) -> Result<Response, String> {
        self.exchange(request)
            .map_err(|e| format!("Server connection: {e}"))
    }
    fn exchange(&self, request: &Request) -> Result<Response, String> {
        let deadline = Instant::now() + self.timeout;
        let mut socket = TcpStream::connect_timeout(
            &self.endpoint.address.into(),
            self.timeout.min(Duration::from_secs(5)),
        )
        .map_err(|e| e.to_string())?;
        socket
            .set_write_timeout(Some(self.timeout.min(Duration::from_secs(5))))
            .map_err(|e| e.to_string())?;
        let (method, path, body) = request.wire();
        if body.len() > 4096 {
            return Err("Request too large".into());
        }
        socket.write_all(format!("{method} {path} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nAccept: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{body}",self.endpoint.address,body.len()).as_bytes()).map_err(|e|e.to_string())?;
        let mut bytes = Vec::new();
        let mut headers = None;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("Request timed out")?;
            socket
                .set_read_timeout(Some(remaining))
                .map_err(|e| e.to_string())?;
            let mut buf = [0; 4096];
            let count = socket.read(&mut buf).map_err(|e| e.to_string())?;
            if count == 0 {
                break;
            }
            bytes.extend_from_slice(&buf[..count]);
            if bytes.len() > MAX_RESPONSE + 16 * 1024 {
                return Err("Response too large".into());
            }
            if headers.is_none() {
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    headers = Some(parse_headers(&bytes[..end], end + 4)?);
                } else if bytes.len() > 16 * 1024 {
                    return Err("HTTP headers too large".into());
                }
            }
            if let Some((_, offset, Some(length), false)) = headers {
                if bytes.len() >= offset + length {
                    break;
                }
            }
        }
        let (status, offset, length, chunked) = headers.ok_or("Incomplete HTTP headers")?;
        let data = &bytes[offset..];
        let body = if chunked {
            decode_chunks(data)?
        } else {
            if length.is_some_and(|n| n != data.len()) {
                return Err("Incomplete HTTP response".into());
            }
            data.to_vec()
        };
        parse_response(request, status, &body)
    }
}
impl PlayClient for HttpClient {
    fn execute(&self, request: &Request) -> Result<Response, String> {
        self.execute(request)
    }
}
fn parse_headers(bytes: &[u8], offset: usize) -> Result<(u16, usize, Option<usize>, bool), String> {
    let text = std::str::from_utf8(bytes).map_err(|_| "Invalid HTTP headers")?;
    let mut lines = text.split("\r\n");
    let mut status = lines.next().ok_or("Missing status")?.split_whitespace();
    if !matches!(status.next(), Some("HTTP/1.1" | "HTTP/1.0")) {
        return Err("Invalid HTTP version".into());
    }
    let status: u16 = status
        .next()
        .ok_or("Missing status")?
        .parse()
        .map_err(|_| "Invalid HTTP status")?;
    let mut length = None;
    let mut chunked = false;
    let mut json = false;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or("Invalid HTTP header")?;
        let value = value.trim();
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some() {
                return Err("Duplicate length".into());
            }
            let n = value
                .parse::<usize>()
                .map_err(|_| "Invalid response length")?;
            if n > MAX_RESPONSE {
                return Err("Response too large".into());
            }
            length = Some(n);
        }
        if name.eq_ignore_ascii_case("transfer-encoding") {
            if !value.eq_ignore_ascii_case("chunked") {
                return Err("Unsupported transfer encoding".into());
            }
            chunked = true;
        }
        if name.eq_ignore_ascii_case("content-type") {
            json = value
                .split(';')
                .next()
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("application/json"));
        }
    }
    if !json || (chunked && length.is_some()) {
        return Err("Invalid JSON HTTP framing".into());
    }
    Ok((status, offset, length, chunked))
}
fn decode_chunks(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut body = Vec::new();
    loop {
        let end = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("Incomplete chunk")?;
        let size = std::str::from_utf8(&data[..end])
            .map_err(|_| "Invalid chunk")?
            .split(';')
            .next()
            .ok_or("Invalid chunk")?;
        let size = usize::from_str_radix(size, 16).map_err(|_| "Invalid chunk size")?;
        data = &data[end + 2..];
        if size == 0 {
            if !data.starts_with(b"\r\n") {
                return Err("Incomplete chunk terminator".into());
            }
            return Ok(body);
        }
        if size > MAX_RESPONSE - body.len()
            || data.len() < size + 2
            || &data[size..size + 2] != b"\r\n"
        {
            return Err("Invalid or oversized chunk".into());
        }
        body.extend_from_slice(&data[..size]);
        data = &data[size + 2..];
    }
}
