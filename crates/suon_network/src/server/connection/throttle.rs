use bevy::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Instant,
};
use thiserror::Error;

use crate::server::settings::{Settings, ThrottlePolicy};

/// Tracks the connection history and block status for a single address.
#[derive(Debug)]
struct State {
    /// Queue of timestamps for recent connection attempts.
    attempts: VecDeque<Instant>,

    /// Optional timestamp until which the address is currently blocked.
    blocked_until: Option<Instant>,

    /// Number of penalties applied (used for calculating exponential backoff).
    penalty_count: u32,

    /// Timestamp of the last observed connection attempt.
    last_seen: Instant,
}

impl State {
    /// Creates a new `State` with an initial timestamp.
    fn new(now: Instant) -> Self {
        Self {
            attempts: VecDeque::new(),
            blocked_until: None,
            penalty_count: 0,
            last_seen: now,
        }
    }
}

/// Errors that can occur while attempting to establish a new connection.
///
/// These errors represent various conditions that can prevent a client
/// from connecting, such as temporary blocking, rate limiting, or internal
/// synchronization failures.
#[derive(Debug, Error, PartialEq, Eq)]
pub(crate) enum AttemptError {
    /// The client is temporarily blocked from initiating new connections.
    ///
    /// This typically occurs when too many failed attempts were made in a short
    /// period, triggering a cooldown period enforced until the specified timestamp.
    #[error("connection attempt blocked until {until:?}")]
    Blocked {
        /// The timestamp until which the client remains blocked.
        until: Instant,
    },

    /// A new connection attempt was made too soon after the previous one.
    ///
    /// This indicates that the client is exceeding the allowed connection rate
    /// and must wait before retrying.
    #[error("connection attempt was made too quickly after the previous one")]
    TooFast,

    /// Failed to acquire the lock protecting the internal state.
    ///
    /// This may happen if another thread or task holds the lock for too long,
    /// or if the lock has been poisoned due to a previous panic.
    #[error("failed to acquire the internal state lock")]
    LockFailed,
}

/// Manages connection throttle and abuse prevention across multiple clients.
#[derive(Resource, Clone)]
pub(crate) struct Throttle {
    /// Shared map of client addresses to their connection states.
    inner: Arc<Mutex<HashMap<SocketAddr, State>>>,

    /// Policy that defines thresholds, limits and backoff durations.
    policy: ThrottlePolicy,
}

impl Throttle {
    /// Creates a new `Throttle` with the specified policy.
    pub fn new(settings: Settings) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            policy: settings.throttle_policy,
        }
    }

    /// Attempts a connection from the specified client address.
    pub fn attempt_connection(&self, addr: &SocketAddr) -> Result<(), AttemptError> {
        let now = Instant::now();

        let mut addresses = self.inner.lock().map_err(|_| {
            warn!("Failed to acquire lock for throttle state");
            AttemptError::LockFailed
        })?;

        let state = addresses.entry(*addr).or_insert_with(|| State::new(now));
        state.last_seen = now;

        // Check if address is currently blocked
        if let Some(until) = state.blocked_until {
            if now < until {
                // Extend block duration with exponential backoff
                let backoff_multiplier = 2u32.pow(state.penalty_count);
                let backoff_duration = self.policy.penalty_backoff * backoff_multiplier;
                let extended_until = until + backoff_duration;
                state.blocked_until = Some(extended_until);

                warn!(
                    "Connection from {addr} blocked until {:?} (extended with backoff)",
                    extended_until
                );

                return Err(AttemptError::Blocked {
                    until: extended_until,
                });
            } else {
                // Block period has expired
                state.blocked_until = None;

                debug!("Block expired for {addr}, allowing new attempts");
            }
        }

        // Remove outdated attempts outside the window
        let window = self.policy.interval_window;
        while let Some(&front) = state.attempts.front() {
            if now.duration_since(front) > window {
                state.attempts.pop_front();
            } else {
                break;
            }
        }

        // Check if the last attempt was too quick
        let fast_threshold = self.policy.fast_attempt_threshold;
        let is_fast = state
            .attempts
            .back()
            .map(|&last_attempt| now.duration_since(last_attempt) <= fast_threshold)
            .unwrap_or(false);

        // Record the new attempt
        state.attempts.push_back(now);

        // Check if attempts exceed the maximum allowed
        if state.attempts.len() > self.policy.max_attempts {
            // Calculate exponential backoff for block duration
            let backoff_multiplier = 2u32.pow(state.penalty_count);
            let backoff_duration = self.policy.block_duration * backoff_multiplier;
            let block_until = now + backoff_duration;

            // Apply block
            state.blocked_until = Some(block_until);
            state.penalty_count = state.penalty_count.saturating_add(1);
            // Optionally clear attempts on block
            state.attempts.clear();

            warn!(
                "Connection from {addr} blocked due to too many attempts. Block until {:?}",
                block_until
            );

            return Err(AttemptError::Blocked { until: block_until });
        }

        // Return error if attempt was too fast
        if is_fast {
            debug!("Connection attempt from {addr} was too fast");
            Err(AttemptError::TooFast)
        } else {
            info!("Connection attempt from {addr} allowed");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        time::Duration,
    };

    const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    #[test]
    fn test_first_attempt_succeeds() {
        // Create a new throttle with the default policy.
        let throttle = Throttle::new(Settings::default());

        // Record the first connection attempt from this address.
        let result = throttle.attempt_connection(&ADDRESS);

        // Verify that the first attempt is always allowed.
        assert!(
            result.is_ok(),
            "The first attempt should always succeed for a new address"
        );
    }

    #[test]
    fn test_fast_repeated_attempt_returns_fast_attempt_error() {
        // Create a new throttle with the default policy.
        let throttle = Throttle::new(Settings::default());

        // Perform the first valid attempt.
        assert!(
            throttle.attempt_connection(&ADDRESS).is_ok(),
            "Initial attempt should be allowed"
        );

        // Perform a second attempt immediately after.
        let result = throttle.attempt_connection(&ADDRESS);

        // Verify that the second attempt is considered too fast.
        assert_eq!(
            result,
            Err(AttemptError::TooFast),
            "A fast repeated attempt should return AttemptError::TooFast"
        );
    }

    #[test]
    fn test_exceeding_max_attempts_blocks_address() {
        // Create a new throttle with the default policy.
        let throttle = Throttle::new(Settings::default());

        // Perform the maximum allowed attempts within the allowed window.
        for _ in 0..throttle.policy.max_attempts {
            assert!(
                throttle.attempt_connection(&ADDRESS).is_ok(),
                "Attempt within the limit should succeed"
            );

            // Wait slightly longer than the fast threshold to avoid triggering TooFast.
            std::thread::sleep(throttle.policy.fast_attempt_threshold + Duration::from_millis(10));
        }

        // Perform one more attempt beyond the limit.
        let result = throttle.attempt_connection(&ADDRESS);

        // Verify that the address becomes blocked.
        assert!(
            matches!(result, Err(AttemptError::Blocked { .. })),
            "Exceeding the maximum allowed attempts should block the address"
        );
    }

    #[test]
    fn test_blocked_address_extends_block_duration_on_repeated_attempts() {
        // Create a new throttle with the default policy.
        let throttle = Throttle::new(Settings::default());

        // Repeatedly attempt connections until the address is blocked.
        for _ in 0..=throttle.policy.max_attempts {
            let _ = throttle.attempt_connection(&ADDRESS);
            std::thread::sleep(throttle.policy.fast_attempt_threshold + Duration::from_millis(10));
        }

        // Record the first blocked attempt.
        let first_block_result = throttle.attempt_connection(&ADDRESS);
        let until_first = match first_block_result {
            Err(AttemptError::Blocked { until }) => until,
            _ => panic!("Expected the first blocked attempt to return AttemptError::Blocked"),
        };

        // Immediately attempt again while still blocked.
        let second_block_result = throttle.attempt_connection(&ADDRESS);
        let until_second = match second_block_result {
            Err(AttemptError::Blocked { until }) => until,
            _ => panic!("Expected the second blocked attempt to return AttemptError::Blocked"),
        };

        // Verify that the second blocked attempt extended the block duration.
        assert!(
            until_second > until_first,
            "The block duration should be extended on repeated blocked attempts"
        );
    }
}
