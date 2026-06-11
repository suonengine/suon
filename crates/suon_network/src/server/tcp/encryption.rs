use serde::{Deserialize, Serialize};

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct EncryptionSettings {
    #[serde(default = "default_true")]
    pub incoming: bool,
    #[serde(default = "default_true")]
    pub outgoing: bool,
}

impl Default for EncryptionSettings {
    fn default() -> Self {
        EncryptionSettings {
            incoming: true,
            outgoing: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encryption_settings_default() {
        let settings = EncryptionSettings::default();
        assert!(settings.incoming);
        assert!(settings.outgoing);
    }

    #[test]
    fn encryption_settings_custom() {
        let settings = EncryptionSettings {
            incoming: false,
            outgoing: true,
        };
        assert!(!settings.incoming);
        assert!(settings.outgoing);
    }
}
