use std::sync::atomic::{AtomicU64, Ordering};

/// Aggregate statistics about all connections managed by a
/// [`ConnectionManager`].
#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub total_accepted: AtomicU64,
    pub total_closed: AtomicU64,
    pub bytes_received: AtomicU64,
    pub bytes_sent: AtomicU64,
}

impl ConnectionStats {
    pub fn record_accepted(&self) {
        self.total_accepted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_closed(&self) {
        self.total_closed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_bytes_received(&self, n: u64) {
        self.bytes_received.fetch_add(n, Ordering::Relaxed);
    }

    pub fn record_bytes_sent(&self, n: u64) {
        self.bytes_sent.fetch_add(n, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stats_initial_values() {
        let stats = ConnectionStats::default();
        assert_eq!(stats.total_accepted.load(Ordering::Relaxed), 0);
        assert_eq!(stats.total_closed.load(Ordering::Relaxed), 0);
        assert_eq!(stats.bytes_received.load(Ordering::Relaxed), 0);
        assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn stats_record_accepted() {
        let stats = ConnectionStats::default();
        stats.record_accepted();
        assert_eq!(stats.total_accepted.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn stats_record_closed() {
        let stats = ConnectionStats::default();
        stats.record_closed();
        assert_eq!(stats.total_closed.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn stats_record_bytes() {
        let stats = ConnectionStats::default();
        stats.record_bytes_received(1024);
        stats.record_bytes_sent(2048);
        assert_eq!(stats.bytes_received.load(Ordering::Relaxed), 1024);
        assert_eq!(stats.bytes_sent.load(Ordering::Relaxed), 2048);
    }

    #[test]
    fn stats_record_multiple() {
        let stats = ConnectionStats::default();
        for _ in 0..100 {
            stats.record_accepted();
        }
        assert_eq!(stats.total_accepted.load(Ordering::Relaxed), 100);
    }
}
