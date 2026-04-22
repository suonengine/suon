use anyhow::Context;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::sync::{Mutex, OnceLock};
use std::{
    fs::{self, File},
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    path::Path,
    time::Duration,
};
use suon_serde::{DocumentedToml, prelude::*};

#[cfg(test)]
pub(crate) fn test_cwd_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Network server configuration.
#[derive(Resource, Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug)]
pub(crate) struct Settings {
    /// IP address and port where the login/game server will listen.
    pub address: SocketAddr,

    /// Enables packet coalescing at the TCP layer.
    /// Keep this disabled for lower latency; enable it only if batching is more important.
    pub use_nagle_algorithm: bool,

    /// Limits for how many clients may stay connected at once.
    pub session_quota: SessionQuota,

    /// Protection against reconnect spam and abusive connection bursts.
    pub throttle_policy: ThrottlePolicy,

    /// Limits for oversized packets and packet floods.
    pub packet_policy: PacketPolicy,
}

impl Settings {
    /// Path to the configuration file.
    const PATH: &'static str = "settings/NetworkServerSettings.toml";

    /// Tries to load the settings, or creates the file with default settings if it doesn't exist.
    pub(crate) fn load_or_default() -> anyhow::Result<Self> {
        let path = Path::new(Self::PATH);

        if path.exists() {
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
        Self::load_from(Path::new(Self::PATH))
    }

    fn load_from(path: &Path) -> anyhow::Result<Self> {
        debug!("Attempting to read configuration from '{}'", path.display());

        let config_str =
            fs::read_to_string(path).context("Failed to read the configuration file")?;

        info!("Successfully read configuration file '{}'", path.display());

        let service_settings = toml::from_str(&config_str)
            .context("Failed to parse the configuration file as TOML")?;

        trace!("Loaded settings: {:?}", service_settings);

        Ok(service_settings)
    }

    /// Creates the configuration file with default settings if it does not exist.
    fn create() -> anyhow::Result<Self> {
        info!("Creating default configuration file '{}'", Self::PATH);

        let default_config = Self::default();

        Self::write_at(Path::new(Self::PATH), &default_config)?;

        info!(
            "Default configuration written to '{}'. Reloading from file.",
            Self::PATH
        );

        // After creating the file, load the settings
        Self::load()
    }

    fn write_at(path: &Path, settings: &Self) -> anyhow::Result<()> {
        debug!("Rendering documented configuration");

        let config_str: String =
            write_documented_toml(settings).context("Failed to serialize default configuration")?;

        debug!("Creating configuration file at '{}'", path.display());

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create settings directory")?;
        }

        let mut file = File::create(path).context("Failed to create the configuration file")?;

        debug!("Writing default configuration to file");

        file.write_all(config_str.as_bytes())
            .context("Failed to write the default configuration to the file")?;

        Ok(())
    }

    /// Returns a log-safe summary of the listener and traffic policies.
    pub(crate) fn summary(self) -> String {
        format!(
            "listen_address={}, nagle={}, session_quota_total={}, session_quota_per_address={}, \
             incoming_timeout_ms={}, outgoing_timeout_ms={}, incoming_max_packet_bytes={}, \
             outgoing_max_packet_bytes={}",
            self.address,
            self.use_nagle_algorithm,
            self.session_quota.max_total,
            self.session_quota.max_per_address,
            self.packet_policy.incoming.timeout.as_millis(),
            self.packet_policy.outgoing.timeout.as_millis(),
            self.packet_policy.incoming.subsequent_max_length,
            self.packet_policy.outgoing.max_length
        )
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

/// Connection caps for the whole server and for a single remote address.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug)]
pub struct SessionQuota {
    /// Total number of active sessions allowed across the server.
    pub max_total: usize,

    /// Maximum simultaneous sessions allowed from the same IP address.
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

/// Rules for slowing down fast reconnects and repeated abusive attempts.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug)]
pub struct ThrottlePolicy {
    /// Maximum connection attempts allowed inside the interval below.
    pub max_attempts: usize,

    /// Rolling time window used when counting connection attempts.
    #[serde(rename = "interval_window_millis", with = "as_millis")]
    pub interval_window: Duration,

    /// Attempts faster than this are treated as suspiciously rapid reconnects.
    #[serde(rename = "fast_attempt_threshold_millis", with = "as_millis")]
    pub fast_attempt_threshold: Duration,

    /// Base block duration applied after abuse is detected.
    #[serde(rename = "block_duration_millis", with = "as_millis")]
    pub block_duration: Duration,

    /// Extra time added to the block for repeated abuse.
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
/// Packet size and timeout rules for traffic entering and leaving the server.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug, Default)]
pub struct PacketPolicy {
    /// Limits applied to packets received from clients.
    pub incoming: IncomingPacketPolicy,

    /// Limits applied to packets sent by the server.
    pub outgoing: OutgoingPacketPolicy,
}

/// Limits for client packet size, receive timeouts, and flood control.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug)]
pub struct IncomingPacketPolicy {
    /// Maximum time to wait for a client packet before the read is considered stalled.
    #[serde(rename = "timeout_millis", with = "as_millis")]
    pub timeout: Duration,

    /// Maximum size accepted for the initial server-name packet.
    pub server_name_max_length: usize,

    /// Maximum size accepted for the login packet.
    pub login_max_length: usize,

    /// Maximum size accepted for packets after login.
    pub subsequent_max_length: usize,

    /// Packet count allowed from one address inside the enforcement window.
    pub subsequent_max_per_address: usize,

    /// Rolling window used for packet flood detection.
    #[serde(rename = "enforcement_window_millis", with = "as_millis")]
    pub enforcement_window: Duration,

    /// Extra packets tolerated above the configured rate before the penalty is applied.
    pub tolerance_overflow: usize,

    /// What to do after a client exceeds the tolerated packet limit.
    /// Available values are `Disconnect` and `Ignore`.
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

/// Action taken when packet flood protection triggers.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Copy, Debug)]
pub enum PacketPolicyPenalty {
    /// Immediately disconnect the offending session.
    Disconnect,

    /// Drop excessive packets and keep the session connected.
    Ignore,
}

/// Limits for packets written by the server.
#[derive(Serialize, Deserialize, DocumentedToml, Clone, Copy, Debug)]
pub struct OutgoingPacketPolicy {
    /// Maximum time to wait while sending a packet to a client.
    #[serde(rename = "timeout_millis", with = "as_millis")]
    pub timeout: Duration,

    /// Maximum size the server is allowed to send in a single packet.
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
        time::{SystemTime, UNIX_EPOCH},
    };

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
        let _lock = test_cwd_lock()
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
        let _lock = test_cwd_lock()
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
