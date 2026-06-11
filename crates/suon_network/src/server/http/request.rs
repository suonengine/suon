use tokio::sync::oneshot;

use suon_channel::IntoTask;

use super::task::HttpRequestTask;

pub(crate) struct HttpRequest {
    pub request_id: u64,
    pub method: String,
    pub path: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub response_sender: oneshot::Sender<Vec<u8>>,
}

impl IntoTask for HttpRequest {
    type Task = HttpRequestTask;

    fn into_task(self) -> HttpRequestTask {
        HttpRequestTask(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::http::task::HttpRequestTask;

    #[test]
    fn http_request_construct() {
        let (tx, _) = oneshot::channel();
        let request = HttpRequest {
            request_id: 1,
            method: "GET".into(),
            path: "/".into(),
            headers: vec![("Host".into(), "localhost".into())],
            body: vec![],
            response_sender: tx,
        };
        assert_eq!(request.request_id, 1);
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/");
        assert_eq!(request.headers.len(), 1);
    }

    #[test]
    fn http_request_into_task() {
        let (tx, _) = oneshot::channel();
        let request = HttpRequest {
            request_id: 1,
            method: "GET".into(),
            path: "/".into(),
            headers: vec![],
            body: vec![],
            response_sender: tx,
        };
        let task: HttpRequestTask = request.into_task();
        assert_eq!(task.0.request_id, 1);
    }
}
