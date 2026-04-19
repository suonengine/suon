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
use suon_serde::prelude::*;

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
    const PATH: &'static str = "settings/NetworkServerSettings.toml";

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

        if let Some(parent) = std::path::Path::new(Self::PATH).parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        path::PathBuf,
        process,
        sync::{Mutex, OnceLock},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn cwd_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct CurrentDirGuard {
        previous: PathBuf,
    }

    impl CurrentDirGuard {
        fn enter(path: &PathBuf) -> Self {
            let previous = env::current_dir().expect("The test should read the current directory");
            env::set_current_dir(path).expect("The test should switch into the temp directory");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            env::set_current_dir(&self.previous)
                .expect("The test should restore the previous current directory");
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time should be after the unix epoch")
            .as_nanos();

        env::temp_dir().join(format!("suon-network-settings-{}-{nanos}", process::id()))
    }

    #[test]
    fn default_settings_use_expected_localhost_address_and_port() {
        let settings = Settings::default();

        assert_eq!(
            settings.address,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 7172)),
            "Default settings should bind to the expected localhost address"
        );
    }

    #[test]
    fn settings_roundtrip_through_toml() {
        let settings = Settings::default();
        let serialized =
            toml::to_string(&settings).expect("Default settings should serialize to TOML");
        let deserialized: Settings =
            toml::from_str(&serialized).expect("Serialized settings should parse back");

        assert_eq!(
            deserialized.address, settings.address,
            "Serialized settings should preserve the bind address"
        );

        assert_eq!(
            deserialized.use_nagle_algorithm, settings.use_nagle_algorithm,
            "Serialized settings should preserve Nagle configuration"
        );

        assert_eq!(
            deserialized.session_quota.max_total, settings.session_quota.max_total,
            "Serialized settings should preserve the session quota"
        );
    }

    #[test]
    fn should_expose_expected_default_session_quota() {
        let quota = SessionQuota::default();

        assert_eq!(
            quota.max_total, 50,
            "The default total session quota should match the intended server capacity"
        );
        assert_eq!(
            quota.max_per_address, 2,
            "The default per-address quota should keep individual clients bounded"
        );
    }

    #[test]
    fn should_expose_expected_default_packet_policies() {
        let policy = PacketPolicy::default();

        assert_eq!(
            policy.incoming.server_name_max_length, 256,
            "The default incoming server-name limit should match the configured protocol bound"
        );
        assert_eq!(
            policy.outgoing.max_length,
            24 * 1024,
            "The default outgoing packet limit should match the expected max payload size"
        );
        assert_eq!(
            policy.incoming.overflow_penalty,
            PacketPolicyPenalty::Disconnect,
            "Incoming packet overflow should default to disconnecting abusive sessions"
        );
    }

    #[test]
    fn load_or_default_should_create_the_configuration_file_when_it_is_missing() {
        let _lock = cwd_lock()
            .lock()
            .expect("The settings test should acquire the cwd lock");
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("The temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let settings = Settings::load_or_default()
            .expect("load_or_default should create and load the default settings");

        assert!(
            temp_dir.join(Settings::PATH).exists(),
            "load_or_default should create the settings file when it does not exist"
        );
        assert_eq!(
            settings.address,
            Settings::default().address,
            "The created configuration should match the default settings"
        );
    }

    #[test]
    fn load_or_default_should_load_an_existing_configuration_file() {
        let _lock = cwd_lock()
            .lock()
            .expect("The settings test should acquire the cwd lock");
        let temp_dir = unique_temp_dir();
        fs::create_dir_all(&temp_dir).expect("The temp test directory should be created");
        let _cwd_guard = CurrentDirGuard::enter(&temp_dir);

        let expected = Settings {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 5), 9000)),
            use_nagle_algorithm: true,
            session_quota: SessionQuota {
                max_total: 3,
                max_per_address: 1,
            },
            throttle_policy: ThrottlePolicy {
                max_attempts: 2,
                interval_window: Duration::from_millis(1234),
                fast_attempt_threshold: Duration::from_millis(55),
                block_duration: Duration::from_millis(777),
                penalty_backoff: Duration::from_millis(99),
            },
            packet_policy: PacketPolicy {
                incoming: IncomingPacketPolicy {
                    timeout: Duration::from_millis(333),
                    server_name_max_length: 11,
                    login_max_length: 22,
                    subsequent_max_length: 33,
                    subsequent_max_per_address: 44,
                    enforcement_window: Duration::from_millis(555),
                    tolerance_overflow: 6,
                    overflow_penalty: PacketPolicyPenalty::Ignore,
                },
                outgoing: OutgoingPacketPolicy {
                    timeout: Duration::from_millis(444),
                    max_length: 88,
                },
            },
        };

        let settings_path = temp_dir.join(Settings::PATH);

        fs::create_dir_all(settings_path.parent().unwrap())
            .expect("The test should create the settings directory");

        fs::write(
            &settings_path,
            toml::to_string_pretty(&expected)
                .expect("The expected settings should serialize to TOML"),
        )
        .expect("The test should write a custom settings file");

        let loaded = Settings::load_or_default()
            .expect("load_or_default should load the existing configuration");

        assert_eq!(
            loaded.address, expected.address,
            "load_or_default should preserve the configured bind address"
        );
        assert_eq!(
            loaded.use_nagle_algorithm, expected.use_nagle_algorithm,
            "load_or_default should preserve the configured Nagle setting"
        );
        assert_eq!(
            loaded.session_quota.max_total, expected.session_quota.max_total,
            "load_or_default should preserve the configured total session quota"
        );
        assert_eq!(
            loaded.packet_policy.incoming.overflow_penalty,
            expected.packet_policy.incoming.overflow_penalty,
            "load_or_default should preserve the configured incoming overflow policy"
        );
        assert_eq!(
            loaded.packet_policy.outgoing.max_length, expected.packet_policy.outgoing.max_length,
            "load_or_default should preserve the configured outgoing packet limit"
        );
    }
}
