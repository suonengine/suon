use parking_lot::Mutex;
use suon_channel::TaskHandler;
use suon_lua::LuaVm;
use suon_macros::Task;
use suon_resource::Resources;
use tokio::sync::oneshot;
use tracing::error;

#[derive(Task)]
pub(crate) struct HttpRequest {
    pub request_id: u64,
    pub port: u16,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub response_sender: Option<oneshot::Sender<Vec<u8>>>,
}

impl TaskHandler for HttpRequest {
    fn run(&mut self, resources: &mut Resources) {
        let vm = resources.get::<LuaVm>();

        let request_id = self.request_id;
        let port = self.port;
        let method = std::mem::take(&mut self.method);
        let path = std::mem::take(&mut self.path);
        let headers = std::mem::take(&mut self.headers);
        let body_bytes = std::mem::take(&mut self.body);
        let body = String::from_utf8_lossy(&body_bytes).to_string();
        let mut maybe_sender = self.response_sender.take();

        vm.execute(|lua| {
            let Some(sender) = maybe_sender.take() else {
                return;
            };

            let Ok(headers_table) = lua.create_table() else {
                return;
            };

            for (key, value) in &headers {
                if let Err(err) = headers_table.set(key.as_str(), value.as_str()) {
                    error!(target: "HTTP", "Failed to set headers table: {err}");
                }
            }

            let sender_mutex = Mutex::new(Some(sender));
            let respond =
                match lua.create_function(move |_, (status, response_body): (u16, String)| {
                    let mut guard = sender_mutex.lock();
                    if let Some(sender) = guard.take() {
                        let status_name = match status {
                            200 => "OK",
                            201 => "Created",
                            204 => "No Content",
                            301 => "Moved Permanently",
                            302 => "Found",
                            304 => "Not Modified",
                            307 => "Temporary Redirect",
                            308 => "Permanent Redirect",
                            400 => "Bad Request",
                            401 => "Unauthorized",
                            402 => "Payment Required",
                            403 => "Forbidden",
                            404 => "Not Found",
                            406 => "Not Acceptable",
                            409 => "Conflict",
                            410 => "Gone",
                            415 => "Unsupported Media Type",
                            422 => "Unprocessable Entity",
                            429 => "Too Many Requests",
                            500 => "Internal Server Error",
                            502 => "Bad Gateway",
                            503 => "Service Unavailable",
                            _ => "Unknown",
                        };

                        let response = format!(
                            "HTTP/1.1 {status} {status_name}\r\nContent-Length: {}\r\n\r\n{}",
                            response_body.len(),
                            response_body
                        );

                        sender.send(response.into_bytes()).ok();
                    }
                    Ok(())
                }) {
                    Ok(func) => func,
                    Err(err) => {
                        error!(target: "HTTP", "Failed to create respond function: {err}");
                        return;
                    }
                };

            let result = vm.trigger_event(
                "RawHttpRequestEvent",
                (request_id, port, method, path, headers_table, body, respond),
            );

            if let Err(err) = result {
                error!(target: "HTTP", "RawHttpRequestEvent error: {err:?}");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    #[test]
    fn http_request_construct() {
        let (tx, _) = oneshot::channel();
        let request = HttpRequest {
            request_id: 1,
            port: 8080,
            method: "GET".into(),
            path: "/".into(),
            headers: vec![("Host".into(), "localhost".into())],
            body: vec![],
            response_sender: Some(tx),
        };
        assert_eq!(request.request_id, 1);
        assert_eq!(request.port, 8080);
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 1);
    }

    #[test]
    fn http_request_run_does_not_panic() {
        let (tx, _) = oneshot::channel();
        let mut resources = suon_resource::Resources::default();
        resources.insert(suon_lua::LuaVm::new());

        let mut task = Box::new(HttpRequest {
            request_id: 1,
            port: 8080,
            method: "GET".into(),
            path: "/".into(),
            headers: vec![],
            body: vec![],
            response_sender: Some(tx),
        });
        task.run(&mut resources);
    }
}
