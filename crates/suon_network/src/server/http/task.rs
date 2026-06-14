use suon_channel::TaskHandler;
use suon_resource::Resources;

use super::request::HttpRequest;

#[allow(dead_code)]
pub(crate) struct HttpRequestTask(pub HttpRequest);

impl TaskHandler for HttpRequestTask {
    fn run(&mut self, _: &mut Resources) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    #[test]
    fn http_request_task_run_does_not_panic() {
        let (tx, _) = oneshot::channel();
        let mut resources = suon_resource::Resources::default();
        let mut task = Box::new(HttpRequestTask(HttpRequest {
            request_id: 1,
            method: "GET".into(),
            path: "/".into(),
            headers: vec![],
            body: vec![],
            response_sender: tx,
        }));
        task.run(&mut resources);
    }
}
