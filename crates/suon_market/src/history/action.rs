/// Action stored in the market history log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketHistoryAction {
    /// A new offer was created.
    Create,
    /// An existing offer was cancelled.
    Cancel,
    /// An existing offer was accepted, fully or partially.
    Accept,
}

impl std::fmt::Display for MarketHistoryAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Create => "create",
            Self::Cancel => "cancel",
            Self::Accept => "accept",
        };

        f.write_str(value)
    }
}

/// Error returned when parsing a textual history action fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMarketHistoryActionError {
    value: String,
}

impl ParseMarketHistoryActionError {
    /// Returns the unsupported raw action value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ParseMarketHistoryActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported market history action '{}'", self.value)
    }
}

impl std::error::Error for ParseMarketHistoryActionError {}

impl std::str::FromStr for MarketHistoryAction {
    type Err = ParseMarketHistoryActionError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "create" => Ok(Self::Create),
            "cancel" => Ok(Self::Cancel),
            "accept" => Ok(Self::Accept),
            other => Err(ParseMarketHistoryActionError {
                value: other.to_string(),
            }),
        }
    }
}
