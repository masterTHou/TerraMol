use super::app::{Action, App};
use serde_json::json;
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
const MAX_HEADER: usize = 16_384;
const MAX_BODY: usize = 8_192;
pub struct Context {
    pub app: Mutex<App>,
    pub stop: AtomicBool,
    pub activity: Mutex<Instant>,
    pub token: String,
    pub origin: String,
    pub host: String,
}
struct Request {
    method: String,
    path: String,
    host: String,
    origin: Option<String>,
    content_type: Option<String>,
    body: Vec<u8>,
}
fn assign_once<T>(slot: &mut Option<T>, value: T) -> io::Result<()> {
    if slot.replace(value).is_some() {
        Err(io::Error::other("duplicate singleton header"))
    } else {
        Ok(())
    }
}
fn request(stream: &mut impl Read) -> io::Result<Request> {
    let mut data = Vec::new();
    let mut chunk = [0u8; 1024];
    let split = loop {
        if let Some(i) = data.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if data.len() > MAX_HEADER {
            return Err(io::Error::other("request header too large"));
        }
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            return Err(io::Error::other("incomplete request"));
        }
        data.extend_from_slice(&chunk[..n]);
    };
    if split > MAX_HEADER {
        return Err(io::Error::other("request header too large"));
    }
    let header =
        std::str::from_utf8(&data[..split]).map_err(|_| io::Error::other("invalid header"))?;
    let mut lines = header.split("\r\n");
    let fields: Vec<_> = lines.next().unwrap_or("").split_whitespace().collect();
    if fields.len() != 3 || fields[2] != "HTTP/1.1" {
        return Err(io::Error::other("HTTP/1.1 required"));
    }
    let (method, path) = (fields[0].to_owned(), fields[1].to_owned());
    let (mut host, mut origin, mut content_type, mut size) = (None, None, None, None);
    for line in lines.filter(|s| !s.is_empty()) {
        let (k, v) = line
            .split_once(':')
            .ok_or_else(|| io::Error::other("invalid header field"))?;
        let k = k.trim().to_ascii_lowercase();
        let v = v.trim();
        match k.as_str() {
            "host" => assign_once(&mut host, v.to_owned())?,
            "origin" => assign_once(&mut origin, v.to_owned())?,
            "content-type" => assign_once(&mut content_type, v.to_owned())?,
            "content-length" => assign_once(
                &mut size,
                v.parse::<usize>()
                    .map_err(|_| io::Error::other("invalid content length"))?,
            )?,
            "transfer-encoding" => return Err(io::Error::other("chunked requests unsupported")),
            _ => {}
        }
    }
    let n = size.unwrap_or(0);
    if n > MAX_BODY {
        return Err(io::Error::other("body too large"));
    }
    while data.len() - split < n {
        let read = stream.read(&mut chunk)?;
        if read == 0 {
            return Err(io::Error::other("incomplete body"));
        }
        data.extend_from_slice(&chunk[..read]);
    }
    Ok(Request {
        method,
        path,
        host: host.ok_or_else(|| io::Error::other("missing host"))?,
        origin,
        content_type,
        body: data[split..split + n].to_vec(),
    })
}
fn respond(stream: &mut TcpStream, status: u16, kind: &str, body: &[u8]) -> io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        503 => "Service Unavailable",
        _ => "Internal Server Error",
    };
    write!(stream,"HTTP/1.1 {status} {reason}\r\nContent-Type: {kind}\r\nContent-Length: {}\r\nConnection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'none'; object-src 'none'\r\n\r\n",body.len())?;
    stream.write_all(body)
}
fn json_response(
    stream: &mut TcpStream,
    status: u16,
    value: &impl serde::Serialize,
) -> io::Result<()> {
    let bytes = serde_json::to_vec(value).map_err(io::Error::other)?;
    respond(stream, status, "application/json; charset=utf-8", &bytes)
}
pub fn busy(stream: &mut TcpStream) {
    let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
    let _ = respond(stream, 503, "text/plain; charset=utf-8", b"Viewer busy");
}
pub fn handle(mut stream: TcpStream, c: &Arc<Context>) {
    // Accepted sockets inherit nonblocking mode on Windows; workers use bounded blocking I/O.
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(1)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(1)));
    let result = (|| -> io::Result<()> {
        let r = match request(&mut stream) {
            Ok(r) => r,
            Err(_) => return json_response(&mut stream, 400, &json!({"error":"无效或过大的请求"})),
        };
        if r.host != c.host || r.origin.as_ref().is_some_and(|s| s != &c.origin) {
            return json_response(
                &mut stream,
                403,
                &json!({"error":"只接受本地可视化页面的请求"}),
            );
        }
        let prefix = format!("/{}/", c.token);
        let route = r.path.split('?').next().unwrap_or("");
        let Some(path) = route.strip_prefix(&prefix) else {
            return respond(&mut stream, 404, "text/plain", b"Not found");
        };
        *c.activity
            .lock()
            .map_err(|_| io::Error::other("activity lock"))? = Instant::now();
        if r.method == "GET" {
            let asset = match path {
                "" => Some((
                    "text/html; charset=utf-8",
                    include_bytes!("../../viewer/index.html").as_slice(),
                )),
                "app.js" => Some((
                    "text/javascript; charset=utf-8",
                    include_bytes!("../../viewer/app.js").as_slice(),
                )),
                "trajectory.js" => Some((
                    "text/javascript; charset=utf-8",
                    include_bytes!("../../viewer/trajectory.js").as_slice(),
                )),
                "renderer.js" => Some((
                    "text/javascript; charset=utf-8",
                    include_bytes!("../../viewer/renderer.js").as_slice(),
                )),
                "style.css" => Some((
                    "text/css; charset=utf-8",
                    include_bytes!("../../viewer/style.css").as_slice(),
                )),
                "favicon.svg" => Some((
                    "image/svg+xml",
                    include_bytes!("../../viewer/favicon.svg").as_slice(),
                )),
                _ => None,
            };
            if let Some((kind, body)) = asset {
                return respond(&mut stream, 200, kind, body);
            }
            if path == "api/experiment" {
                let app = c.app.lock().map_err(|_| io::Error::other("app lock"))?;
                return json_response(&mut stream, 200, &app.experiment());
            }
            if path == "api/state" {
                let mut app = c.app.lock().map_err(|_| io::Error::other("app lock"))?;
                return match app.state() {
                    Ok(s) => json_response(&mut stream, 200, &s),
                    Err(e) => json_response(&mut stream, 500, &json!({"error":e.message})),
                };
            }
        } else if r.method == "POST" && path == "api/action" {
            if r.content_type.as_deref() != Some("application/json") {
                return json_response(&mut stream, 400, &json!({"error":"需要 JSON 请求"}));
            }
            let action: Action = match serde_json::from_slice(&r.body) {
                Ok(a) => a,
                Err(_) => return json_response(&mut stream, 400, &json!({"error":"操作参数无效"})),
            };
            let mut app = c.app.lock().map_err(|_| io::Error::other("app lock"))?;
            match app.apply(action) {
                Ok(shutdown) => {
                    if shutdown {
                        let result = json_response(&mut stream, 200, &json!({"shutdown":true}));
                        c.stop.store(true, Ordering::Relaxed);
                        return result;
                    }
                    let state = app.state().map_err(io::Error::other)?;
                    let result = json_response(&mut stream, 200, &state);
                    return result;
                }
                Err(e) => {
                    let message = match e.code {
                        crate::error::Code::Capacity => {
                            "空间不足，请删除部分分子或清空场景后重试".to_owned()
                        }
                        crate::error::Code::NotFound => "所选分子已不存在，请重新选择".to_owned(),
                        _ => e.message,
                    };
                    return json_response(&mut stream, 409, &json!({"error":message}));
                }
            }
        } else if r.method != "GET" {
            return respond(&mut stream, 405, "text/plain", b"Method not allowed");
        }
        respond(&mut stream, 404, "text/plain", b"Not found")
    })();
    // A disconnected browser is not a simulation error; resources close with the stream.
    let _ = result;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_framing() {
        for s in [
            "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n",
            "POST / HTTP/1.1\r\nHost: a\r\nTransfer-Encoding: chunked\r\n\r\n",
            "POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 99999\r\n\r\n",
        ] {
            assert!(request(&mut s.as_bytes()).is_err());
        }
    }
}
