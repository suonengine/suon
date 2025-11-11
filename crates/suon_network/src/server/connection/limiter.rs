use bevy::prelude::*;
use std::{collections::HashMap, net::SocketAddr};
use thiserror::Error;

use crate::server::settings::{SessionQuota, Settings};

/// Tracks the number of active sessions for a specific address.
#[derive(Debug)]
struct State {
    active: usize,
}

/// Errors that can occur while attempting to acquire a new session slot.
///
/// These errors represent conditions that prevent the creation of new sessions,
/// such as global or per-address limits being reached. They typically indicate
/// that the system has reached its configured capacity for active connections.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AcquireError {
    /// The total maximum number of concurrent sessions has been reached.
    ///
    /// This occurs when the total number of active sessions in the system
    /// equals or exceeds the configured global limit.
    #[error("total maximum number of sessions ({max_total}) has been reached")]
    TotalReached {
        /// The maximum number of total sessions allowed.
        max_total: usize,
    },

    /// The maximum number of sessions for this specific address has been reached.
    ///
    /// This typically indicates that the client at the given address has opened
    /// too many simultaneous sessions, exceeding the configured per-address limit.
    #[error("maximum sessions per address ({max_per_addr}) reached for {addr}")]
    PerAddressReached {
        /// The address that exceeded its session limit.
        addr: SocketAddr,

        /// The maximum number of sessions allowed per address.
        max_per_addr: usize,
    },
}

/// Manages active sessions and enforces global and per-address session limits.
#[derive(Resource)]
pub struct Limiter {
    /// Current total number of active sessions.
    total_active: usize,

    /// Tracks active sessions per address.
    sessions: HashMap<SocketAddr, State>,

    /// Configuration for session limits.
    session_quota: SessionQuota,
}

impl Limiter {
    /// Creates a new `Limiter` with specified session quotas.
    pub(crate) fn new(settings: Settings) -> Self {
        Self {
            session_quota: settings.session_quota,
            total_active: 0,
            sessions: HashMap::new(),
        }
    }

    /// Attempts to acquire a session slot for the given address.
    pub fn try_acquire(&mut self, addr: SocketAddr) -> Result<(), AcquireError> {
        // Check if total session limit has been reached.
        if self.total_active >= self.session_quota.max_total {
            warn!(
                "Total session limit reached ({} active, max {})",
                self.total_active, self.session_quota.max_total
            );

            return Err(AcquireError::TotalReached {
                max_total: self.session_quota.max_total,
            });
        }

        // Retrieve or initialize session state for the address.
        let state = self.sessions.entry(addr).or_insert(State { active: 0 });

        // Check if this address has reached its session limit.
        if state.active >= self.session_quota.max_per_address {
            warn!(
                "Per-address session limit reached for {addr} ({} active, max {})",
                state.active, self.session_quota.max_per_address
            );

            return Err(AcquireError::PerAddressReached {
                addr,
                max_per_addr: self.session_quota.max_per_address,
            });
        }

        // Increment counters for total and per-address sessions.
        self.total_active += 1;
        state.active += 1;

        debug!(
            "Acquired session for {addr}. Total active: {}, Address active: {}",
            self.total_active, state.active
        );

        Ok(())
    }

    /// Releases a session slot for the given address.
    pub fn release(&mut self, addr: SocketAddr) {
        let Some(state) = self.sessions.get_mut(&addr) else {
            warn!("Attempted to release session for {addr} which has no active sessions",);
            return;
        };

        // Decrement per-address session count.
        if state.active > 0 {
            state.active -= 1;
        }

        // Decrement total session count.
        if self.total_active > 0 {
            self.total_active -= 1;
        }

        debug!(
            "Released session for {addr}. Total active: {}, Address active: {}",
            self.total_active, state.active
        );
    }

    /// Returns the current total number of active sessions.
    pub fn total_active_sessions(&self) -> usize {
        self.total_active
    }

    /// Returns the number of active sessions for a specific address.
    pub fn active_sessions_for_address(&self, addr: SocketAddr) -> usize {
        self.sessions
            .get(&addr)
            .map(|state| state.active)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    impl Limiter {
        fn with_session_quota(session_quota: SessionQuota) -> Self {
            Self {
                session_quota,
                total_active: 0,
                sessions: HashMap::new(),
            }
        }
    }

    const ADDRESS: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080);

    #[test]
    fn first_session_is_successfully_acquired() {
        // Create a session limiter with generous limits.
        let mut limiter = Limiter::with_session_quota(SessionQuota {
            max_total: 10,
            max_per_address: 5,
        });

        // Attempt to acquire the first session.
        let result = limiter.try_acquire(ADDRESS);

        // The acquisition should succeed.
        assert!(
            result.is_ok(),
            "The first session should be acquired successfully"
        );

        // Counters should be updated accordingly.
        assert_eq!(
            limiter.total_active_sessions(),
            1,
            "Total active sessions should be 1"
        );

        assert_eq!(
            limiter.active_sessions_for_address(ADDRESS),
            1,
            "Active sessions for the test address should be 1"
        );
    }

    #[test]
    fn cannot_exceed_total_session_limit() {
        let mut limiter = Limiter::with_session_quota(SessionQuota {
            max_total: 1,
            max_per_address: 10,
        });

        // Acquire the first session successfully.
        assert!(limiter.try_acquire(ADDRESS).is_ok());

        // Next attempt should fail because total limit is reached.
        let result = limiter.try_acquire(ADDRESS);
        assert!(
            matches!(result, Err(AcquireError::TotalReached { .. })),
            "Should return TotalReached when total limit is exceeded"
        );

        // Counters should remain unchanged.
        assert_eq!(
            limiter.total_active_sessions(),
            1,
            "Total sessions should remain 1"
        );

        assert_eq!(
            limiter.active_sessions_for_address(ADDRESS),
            1,
            "Active sessions for the address should remain 1"
        );
    }

    #[test]
    fn cannot_exceed_sessions_per_address_limit() {
        let mut limiter = Limiter::with_session_quota(SessionQuota {
            max_total: 10,
            max_per_address: 2,
        });

        // Acquire sessions up to the per-address limit.
        assert!(limiter.try_acquire(ADDRESS).is_ok());
        assert!(limiter.try_acquire(ADDRESS).is_ok());

        // Next acquisition should fail due to per-address limit.
        let result = limiter.try_acquire(ADDRESS);
        assert!(
            matches!(result, Err(AcquireError::PerAddressReached { .. })),
            "Should return PerAddressReached when per-address limit is exceeded"
        );

        // Counters should reflect the successful acquisitions.
        assert_eq!(
            limiter.total_active_sessions(),
            2,
            "Total sessions should be 2"
        );

        assert_eq!(
            limiter.active_sessions_for_address(ADDRESS),
            2,
            "Active sessions for the address should be 2"
        );
    }

    #[test]
    fn releasing_sessions_updates_counters_correctly() {
        let mut limiter = Limiter::with_session_quota(SessionQuota {
            max_total: 5,
            max_per_address: 3,
        });

        // Acquire two sessions.
        assert!(limiter.try_acquire(ADDRESS).is_ok());
        assert!(limiter.try_acquire(ADDRESS).is_ok());

        // Release one session and check counters.
        limiter.release(ADDRESS);
        assert_eq!(
            limiter.total_active_sessions(),
            1,
            "Total sessions should be 1 after release"
        );

        assert_eq!(
            limiter.active_sessions_for_address(ADDRESS),
            1,
            "Active sessions for the address should be 1 after release"
        );

        // Release the second session.
        limiter.release(ADDRESS);
        assert_eq!(
            limiter.total_active_sessions(),
            0,
            "All sessions should be released"
        );

        assert_eq!(
            limiter.active_sessions_for_address(ADDRESS),
            0,
            "No active sessions should remain for the address"
        );
    }
}
