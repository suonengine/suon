#[derive(Debug)]
pub enum SettingsError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    Validation(String),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::Io(error) => write!(formatter, "{error}"),
            SettingsError::Parse(error) => write!(formatter, "{error}"),
            SettingsError::Serialize(error) => write!(formatter, "{error}"),
            SettingsError::Validation(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for SettingsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            SettingsError::Io(error) => Some(error),
            SettingsError::Parse(error) => Some(error),
            SettingsError::Serialize(error) => Some(error),
            SettingsError::Validation(_) => None,
        }
    }
}

impl From<std::io::Error> for SettingsError {
    fn from(error: std::io::Error) -> Self {
        SettingsError::Io(error)
    }
}

impl From<toml::de::Error> for SettingsError {
    fn from(error: toml::de::Error) -> Self {
        SettingsError::Parse(error)
    }
}

impl From<toml::ser::Error> for SettingsError {
    fn from(error: toml::ser::Error) -> Self {
        SettingsError::Serialize(error)
    }
}
