use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::Path,
    time::Duration,
};
use suon_serde::duration::as_millis;

/// Network server configuration.
#[derive(Resource, Serialize, Deserialize, Clone, Copy, Debug)]
pub(crate) struct Settings {
    /// IP address and port the server will bind to.
    pub address: SocketAddr,

    /// Enable or disable Nagle's algorithm for connections.
    pub use_nagle_algorithm: bool,

    /// Limits on total and per-address simultaneous sessions.
    pub session_quota: SessionQuota,

    /// Policy for managing rapid reconnects and preventing abuse.
    pub throttle_policy: ThrottlePolicy,

    /// Policy for controlling packet floods and excessive network traffic.
    pub packet_policy: PacketPolicy,
}

impl Settings {
    /// Path to the configuration file.
    const PATH: &'static str = "NetworkServerSettings.toml";

    /// Tries to load the settings, or creates the file with default settings if it doesn't exist.
    pub(crate) fn load_or_default() -> anyhow::Result<Self> {
        if Path::new(Self::PATH).exists() {
            info!(
                "Configuration file '{}' found, attempting to load.",
                Self::PATH
            );

            Self::load()
        } else {
            warn!(
                "Configuration file '{}' not found. Creating default configuration.",
                Self::PATH
            );

            Self::create()
        }
    }

    /// Tries to load the settings from the file.
    fn load() -> anyhow::Result<Self> {
        debug!("Attempting to read configuration from '{}'", Self::PATH);

        let config_str =
            fs::read_to_string(Self::PATH).context("Failed to read the configuration file")?;

        info!("Successfully read configuration file '{}'", Self::PATH);

        let service_settings = toml::from_str(&config_str)
            .context("Failed to parse the configuration file as TOML")?;

        trace!("Loaded settings: {:?}", service_settings);

        Ok(service_settings)
    }

    /// Creates the configuration file with default settings if it does not exist.
    fn create() -> anyhow::Result<Self> {
        info!("Creating default configuration file '{}'", Self::PATH);

        let default_config = Self::default();

        debug!("Serializing default configuration to TOML format");

        let config_str: String = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default configuration")?;

        debug!("Creating configuration file at '{}'", Self::PATH);

        let mut file =
            File::create(Self::PATH).context("Failed to create the configuration file")?;

        debug!("Writing default configuration to file");

        file.write_all(config_str.as_bytes())
            .context("Failed to write the default configuration to the file")?;

        info!(
            "Default configuration written to '{}'. Reloading from file.",
            Self::PATH
        );

        // After creating the file, load the settings
        Self::load()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7172)),
            use_nagle_algorithm: false,
            session_quota: SessionQuota::default(),
            throttle_policy: ThrottlePolicy::default(),
            packet_policy: PacketPolicy::default(),
        }
    }
}

/// Configuration for limiting the number of simultaneous sessions.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SessionQuota {
    /// Maximum number of total active sessions allowed at the same time.
    pub max_total: usize,

    /// Maximum number of active sessions allowed per individual address.
    pub max_per_address: usize,
}

impl Default for SessionQuota {
    fn default() -> Self {
        Self {
            max_total: 50,
            max_per_address: 2,
        }
    }
}

/// Configuration for managing connection retries and abuse prevention.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct ThrottlePolicy {
    /// Maximum allowed connection attempts within the specified interval.
    pub max_attempts: usize,

    /// Duration of the interval window for counting connection attempts.
    #[serde(rename = "interval_window_millis", with = "as_millis")]
    pub interval_window: Duration,

    /// Duration considered as a "fast" attempt.
    #[serde(rename = "fast_attempt_threshold_millis", with = "as_millis")]
    pub fast_attempt_threshold: Duration,

    /// Duration for blocking an abusive IP.
    #[serde(rename = "block_duration_millis", with = "as_millis")]
    pub block_duration: Duration,

    /// Additional backoff time added to the block duration for continued abuse.
    #[serde(rename = "penalty_backoff_millis", with = "as_millis")]
    pub penalty_backoff: Duration,
}

impl Default for ThrottlePolicy {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            interval_window: Duration::from_millis(5000),
            fast_attempt_threshold: Duration::from_millis(500),
            block_duration: Duration::from_millis(3000),
            penalty_backoff: Duration::from_millis(250),
        }
    }
}
/// Configuration for controlling packet floods and traffic for both incoming and outgoing packets.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct PacketPolicy {
    /// Flood control settings for incoming packets.
    pub incoming: IncomingPacketPolicy,

    /// Flood control settings for outgoing packets.
    pub outgoing: OutgoingPacketPolicy,
}

/// Policy for controlling floods of incoming packets and enforcing limits per address.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct IncomingPacketPolicy {
    /// Maximum duration to wait for reading from a session.
    #[serde(rename = "timeout_millis", with = "as_millis")]
    pub timeout: Duration,

    /// Maximum allowed length of a single packet.
    pub server_name_max_length: usize,

    /// Maximum allowed length of a single packet.
    pub login_max_length: usize,

    /// Maximum allowed length of a single packet.
    pub subsequent_max_length: usize,

    /// Maximum number of packets allowed per address within the enforcement window.
    pub subsequent_max_per_address: usize,

    /// Duration of the time window in which packet limits are enforced.
    #[serde(rename = "enforcement_window_millis", with = "as_millis")]
    pub enforcement_window: Duration,

    /// Number of packets above the limit tolerated before applying a penalty.
    pub tolerance_overflow: usize,

    /// Action to take when the packet limit is exceeded.
    pub overflow_penalty: PacketPolicyPenalty,
}

impl Default for IncomingPacketPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(30000),
            server_name_max_length: 256,
            login_max_length: 5 * 1024,
            subsequent_max_length: 20 * 1024,
            subsequent_max_per_address: u32::MAX as usize,
            enforcement_window: Duration::from_millis(1000),
            tolerance_overflow: 20,
            overflow_penalty: PacketPolicyPenalty::Disconnect,
        }
    }
}

/// Action to take when a packet limit is exceeded.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PacketPolicyPenalty {
    /// Disconnect the session if the packet limit is exceeded.
    Disconnect,

    /// Ignore excessive packets without disconnecting the session.
    Ignore,
}

/// Policy for controlling floods of outgoing packets.
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct OutgoingPacketPolicy {
    /// Maximum duration to wait for writing to a session.
    #[serde(rename = "timeout_millis", with = "as_millis")]
    pub timeout: Duration,

    /// Maximum allowed length of a single packet.
    pub max_length: usize,
}

impl Default for OutgoingPacketPolicy {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(30000),
            max_length: 24 * 1024,
        }
    }
}
