use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};

use super::{acceptor::HttpSettings, request::HttpRequest};
use crate::server::{shutdown::Shutdown, throttle::ConnectionPermit};

const MAX_HEADERS: usize = 64;
const READ_BUF_SIZE: usize = 4096;
const MAX_BODY_SIZE: usize = 4 * 1024 * 1024;

pub(crate) struct HttpSession {
    request_id: u64,
    stream: TcpStream,
    channel: suon_channel::Channel,
    config: HttpSettings,
    shutdown: Shutdown,
    _permit: ConnectionPermit,
}

impl HttpSession {
    pub fn new(
        request_id: u64,
        stream: TcpStream,
        channel: suon_channel::Channel,
        config: HttpSettings,
        shutdown: Shutdown,
        _permit: ConnectionPermit,
    ) -> Self {
        HttpSession {
            request_id,
            stream,
            channel,
            config,
            shutdown,
            _permit,
        }
    }

    pub fn spawn(self) {
        tokio::spawn(self.run());
    }

    async fn run(mut self) {
        let mut buffer = Vec::with_capacity(READ_BUF_SIZE);

        loop {
            let mut chunk = vec![0u8; READ_BUF_SIZE];
            let bytes_read = match self.stream.read(&mut chunk).await {
                Ok(0) => return,
                Ok(bytes) => bytes,
                Err(error) => {
                    eprintln!("[HTTP] read error: {error}");
                    return;
                }
            };
            buffer.extend_from_slice(&chunk[..bytes_read]);

            if buffer.len() > MAX_BODY_SIZE {
                eprintln!("[HTTP] request too large");
                return;
            }

            if buffer.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let header_end = buffer
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|pos| pos + 4)
            .unwrap_or(0);
        let header_bytes = &buffer[..header_end];

        let content_length = std::str::from_utf8(header_bytes)
            .ok()
            .and_then(|headers| {
                headers
                    .lines()
                    .find_map(|line| {
                        line.strip_prefix("Content-Length:")
                            .or_else(|| line.strip_prefix("content-length:"))
                    })
                    .and_then(|value| value.trim().parse::<usize>().ok())
            })
            .unwrap_or(0);

        if content_length > MAX_BODY_SIZE {
            eprintln!("[HTTP] body too large: {content_length}");
            return;
        }

        let body_end = header_end + content_length;

        while buffer.len() < body_end {
            let remaining = body_end - buffer.len();
            let mut chunk = vec![0u8; remaining.min(READ_BUF_SIZE)];
            let bytes_read = match self.stream.read(&mut chunk).await {
                Ok(0) => return,
                Ok(bytes) => bytes,
                Err(error) => {
                    eprintln!("[HTTP] read error: {error}");
                    return;
                }
            };
            buffer.extend_from_slice(&chunk[..bytes_read]);
        }

        let mut raw_headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
        let mut request = httparse::Request::new(&mut raw_headers);

        let (method, path, headers, body) = match request.parse(&buffer) {
            Ok(status) => {
                let consumed = match status {
                    httparse::Status::Complete(n) => n,
                    httparse::Status::Partial => unreachable!(),
                };

                let method = request.method.unwrap_or("GET").to_string();
                let path = request.path.unwrap_or("/").to_string();
                let header_count = request.headers.len().min(self.config.max_headers);
                let headers: Vec<(String, String)> = raw_headers[..header_count]
                    .iter()
                    .filter_map(|h| {
                        let name = std::str::from_utf8(h.name.as_bytes()).ok()?.to_string();
                        let value = std::str::from_utf8(h.value).ok()?.to_string();
                        Some((name, value))
                    })
                    .collect();

                let body = buffer[consumed..].to_vec();
                (method, path, headers, body)
            }
            Err(e) => {
                eprintln!("[HTTP] parse error: {e}");
                ("GET".into(), "/".into(), Vec::new(), Vec::new())
            }
        };

        let (response_sender, response_receiver) = oneshot::channel::<Vec<u8>>();
        self.channel.send(HttpRequest {
            request_id: self.request_id,
            method,
            path,
            headers,
            body,
            response_sender,
        });

        let mut rx = self.shutdown.receiver();
        tokio::select! {
            _ = rx.changed() => {}
            result = response_receiver => {
                if let Ok(data) = result {
                    if let Err(e) = self.stream.write_all(&data).await { eprintln!("[HTTP] write error: {e}"); }
                    if let Err(e) = self.stream.flush().await { eprintln!("[HTTP] flush error: {e}"); }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::throttle::ConnectionLimiter;
    use suon_channel::Channel;
    use tokio::net::TcpListener;

    fn make_config() -> HttpSettings {
        HttpSettings {
            max_connections: 100,
            rate_burst: 50,
            max_headers: 32,
        }
    }

    #[tokio::test]
    async fn http_session_receives_request() {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind TCP listener for HTTP session test");
        let addr = listener
            .local_addr()
            .expect("failed to get listener local address");
        let channel = Channel::default();
        let shutdown = Shutdown::new();
        let limiter = ConnectionLimiter::new(5);
        let permit = limiter
            .try_acquire()
            .expect("failed to acquire connection permit for test");

        let server = tokio::spawn(async move {
            let (stream, _) = listener
                .accept()
                .await
                .expect("failed to accept incoming connection");
            HttpSession::new(1, stream, channel, make_config(), shutdown, permit).spawn();
        });

        let mut client = tokio::net::TcpStream::connect(addr)
            .await
            .expect("failed to connect test client");
        tokio::io::AsyncWriteExt::write_all(
            &mut client,
            b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
        )
        .await
        .expect("failed to send HTTP request from test client");

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        drop(client);
        drop(server.await);
    }
}
