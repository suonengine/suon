use tokio::sync::watch;

#[derive(Clone)]
pub(crate) struct Shutdown {
    sender: watch::Sender<bool>,
    receiver: watch::Receiver<bool>,
}

impl Shutdown {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(false);
        Shutdown { sender, receiver }
    }

    pub fn trigger(&self) {
        if let Err(e) = self.sender.send(true) {
            eprintln!("[Shutdown] send error: {e}");
        }
    }

    pub fn receiver(&self) -> watch::Receiver<bool> {
        self.receiver.clone()
    }

    pub fn is_triggered(&self) -> bool {
        *self.receiver.borrow()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shutdown_new_is_not_triggered() {
        let shutdown = Shutdown::new();
        assert!(!shutdown.is_triggered());
    }

    #[test]
    fn shutdown_trigger_sets_flag() {
        let shutdown = Shutdown::new();
        shutdown.trigger();
        assert!(shutdown.is_triggered());
    }

    #[test]
    fn shutdown_trigger_multiple_times() {
        let shutdown = Shutdown::new();
        shutdown.trigger();
        shutdown.trigger();
        assert!(shutdown.is_triggered());
    }

    #[test]
    fn shutdown_clone_shares_state() {
        let original = Shutdown::new();
        let cloned = original.clone();
        original.trigger();
        assert!(cloned.is_triggered());
    }

    #[test]
    fn shutdown_receiver_detects_trigger() {
        let shutdown = Shutdown::new();
        let rx = shutdown.receiver();
        shutdown.trigger();
        // receiver sees the latest value
        assert!(*rx.borrow());
    }

    #[test]
    fn shutdown_is_triggered_does_not_consume() {
        let shutdown = Shutdown::new();
        assert!(!shutdown.is_triggered());
        assert!(!shutdown.is_triggered());
        shutdown.trigger();
        assert!(shutdown.is_triggered());
        assert!(shutdown.is_triggered());
    }

    #[test]
    fn shutdown_clone_can_trigger_original() {
        let original = Shutdown::new();
        let cloned = original.clone();
        cloned.trigger();
        assert!(original.is_triggered());
    }
}
