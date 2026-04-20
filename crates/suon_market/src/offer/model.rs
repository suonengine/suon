use std::time::SystemTime;

use suon_protocol_client::prelude::MarketOfferKind;

/// A market actor-name snapshot loaded for lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketActorName {
    id: u32,
    name: String,
}

impl MarketActorName {
    /// Creates a new market actor-name snapshot.
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Returns the actor identifier.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the actor display name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A market item snapshot loaded for market lookups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketItem {
    id: u16,
    name: String,
}

impl MarketItem {
    /// Creates a new market-item snapshot.
    pub fn new(id: u16, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }

    /// Returns the item identifier.
    pub fn id(&self) -> u16 {
        self.id
    }

    /// Returns the item display name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Whether a market offer is buying or selling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MarketTradeSide {
    /// The offer buys items from other actors.
    Buy,
    /// The offer sells items to other actors.
    Sell,
}

/// Error returned when parsing a textual market trade side fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMarketTradeSideError {
    value: String,
}

impl ParseMarketTradeSideError {
    /// Returns the unsupported raw side value.
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ParseMarketTradeSideError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported market trade side '{}'", self.value)
    }
}

impl std::error::Error for ParseMarketTradeSideError {}

impl std::fmt::Display for MarketTradeSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        };

        f.write_str(value)
    }
}

impl std::str::FromStr for MarketTradeSide {
    type Err = ParseMarketTradeSideError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "buy" => Ok(Self::Buy),
            "sell" => Ok(Self::Sell),
            other => Err(ParseMarketTradeSideError {
                value: other.to_string(),
            }),
        }
    }
}

impl From<MarketOfferKind> for MarketTradeSide {
    fn from(value: MarketOfferKind) -> Self {
        match value {
            MarketOfferKind::Buy => Self::Buy,
            MarketOfferKind::Sell => Self::Sell,
        }
    }
}

/// Stable identifier used by the client protocol for existing offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MarketOfferId {
    timestamp: SystemTime,
    counter: u16,
}

impl MarketOfferId {
    /// Creates a new market-offer identifier.
    pub fn new(timestamp: SystemTime, counter: u16) -> Self {
        Self { timestamp, counter }
    }

    /// Returns the protocol timestamp portion of the offer identifier.
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }

    /// Returns the protocol counter portion of the offer identifier.
    pub fn counter(&self) -> u16 {
        self.counter
    }
}

/// Cached market offer row loaded from an ORM source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketOffer {
    id: MarketOfferId,
    item_id: u16,
    actor_id: u32,
    amount: u16,
    price: u64,
    side: MarketTradeSide,
    is_anonymous: bool,
}

impl MarketOffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MarketOfferId,
        item_id: u16,
        actor_id: u32,
        amount: u16,
        price: u64,
        side: MarketTradeSide,
        is_anonymous: bool,
    ) -> Self {
        Self {
            id,
            item_id,
            actor_id,
            amount,
            price,
            side,
            is_anonymous,
        }
    }

    /// Returns the stable market-offer identifier.
    pub fn id(&self) -> MarketOfferId {
        self.id
    }

    /// Returns the offered item identifier.
    pub fn item_id(&self) -> u16 {
        self.item_id
    }

    /// Returns the owning actor identifier.
    pub fn actor_id(&self) -> u32 {
        self.actor_id
    }

    /// Returns the remaining offer amount.
    pub fn amount(&self) -> u16 {
        self.amount
    }

    /// Returns the unit price for the offer.
    pub fn price(&self) -> u64 {
        self.price
    }

    /// Returns whether the offer is a buy or sell entry.
    pub fn side(&self) -> MarketTradeSide {
        self.side
    }

    /// Returns whether the offer should hide its owner identity.
    pub fn is_anonymous(&self) -> bool {
        self.is_anonymous
    }
}
