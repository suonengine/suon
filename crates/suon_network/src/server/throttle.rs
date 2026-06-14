use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use tracing::{debug, trace};

use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

#[derive(Debug, Clone)]
pub(crate) struct ConnectionLimiter {
    semaphore: Arc<Semaphore>,
    active: Arc<AtomicUsize>,
}

impl ConnectionLimiter {
    pub fn new(max: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max)),
            active: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn try_acquire(&self) -> Result<ConnectionPermit, TryAcquireError> {
        let permit = self.semaphore.clone().try_acquire_owned()?;
        self.active.fetch_add(1, Ordering::Relaxed);
        Ok(ConnectionPermit {
            _permit: Some(permit),
            active: self.active.clone(),
        })
    }

    pub fn active_count(&self) -> usize {
        self.active.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub(crate) struct ConnectionPermit {
    _permit: Option<OwnedSemaphorePermit>,
    active: Arc<AtomicUsize>,
}

impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::Relaxed);
    }
}

#[derive(Debug, Default)]
struct PacketCounter {
    timestamps: Vec<Instant>,
}

#[derive(Debug, Clone)]
pub(crate) struct PacketRateLimiter {
    inner: Arc<Mutex<HashMap<SocketAddr, PacketCounter>>>,
    max_burst: u32,
}

impl PacketRateLimiter {
    pub fn new(max_burst: u32) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            max_burst,
        }
    }

    pub fn allow(&self, addr: SocketAddr) -> bool {
        let mut inner = self.inner.lock().unwrap_or_else(|e| e.into_inner());
        let state = inner.entry(addr).or_default();
        let now = Instant::now();

        state
            .timestamps
            .retain(|t| now.duration_since(*t).as_secs() < 1);

        if state.timestamps.len() >= self.max_burst as usize {
            debug!(target: "Throttle", "Rate limiting {addr}: burst exceeded");
            return false;
        }

        state.timestamps.push(now);
        true
    }

    pub fn remove(&self, addr: &SocketAddr) {
        drop(
            self.inner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(addr),
        );
        trace!(target: "Throttle", "Rate limiter removed {addr}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, SocketAddrV4};

    fn test_addr(n: u16) -> SocketAddr {
        SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, n))
    }

    #[tokio::test]
    async fn limiter_acquire_release() {
        let limiter = ConnectionLimiter::new(2);

        let p1 = limiter
            .try_acquire()
            .expect("first test permit should not fail with max=2");

        let p2 = limiter
            .try_acquire()
            .expect("second test permit should not fail with max=2");

        assert!(limiter.try_acquire().is_err());

        assert_eq!(limiter.active_count(), 2);
        drop(p1);
        assert_eq!(limiter.active_count(), 1);

        let p3 = limiter
            .try_acquire()
            .expect("third permit after dropping one should succeed");

        assert!(limiter.try_acquire().is_err());

        drop(p2);
        drop(p3);
        assert_eq!(limiter.active_count(), 0);
    }

    #[tokio::test]
    async fn limiter_zero_max_rejects_all() {
        let limiter = ConnectionLimiter::new(0);
        assert!(limiter.try_acquire().is_err());
        assert_eq!(limiter.active_count(), 0);
    }

    #[test]
    fn rate_limiter_allows_up_to_burst() {
        let rl = PacketRateLimiter::new(3);
        let addr = test_addr(1);

        assert!(rl.allow(addr));
        assert!(rl.allow(addr));
        assert!(rl.allow(addr));
        assert!(!rl.allow(addr));
    }

    #[test]
    fn rate_limiter_zero_burst_blocks_all() {
        let rl = PacketRateLimiter::new(0);
        let addr = test_addr(2);
        assert!(!rl.allow(addr));
    }

    #[test]
    fn rate_limiter_remove_clears_state() {
        let rl = PacketRateLimiter::new(2);
        let addr = test_addr(3);

        assert!(rl.allow(addr));
        assert!(rl.allow(addr));
        assert!(!rl.allow(addr));

        rl.remove(&addr);
        assert!(rl.allow(addr));
    }

    #[test]
    fn rate_limiter_per_ip_independent() {
        let rl = PacketRateLimiter::new(2);
        let a = test_addr(10);
        let b = test_addr(20);

        assert!(rl.allow(a));
        assert!(rl.allow(b));
        assert!(rl.allow(a));
        assert!(rl.allow(b));
        assert!(!rl.allow(a));
        assert!(!rl.allow(b));
    }
}
