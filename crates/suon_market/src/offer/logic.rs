use std::{
    collections::{HashMap, VecDeque},
    num::Wrapping,
    time::{Duration, SystemTime},
};

use bevy::prelude::*;
use log::{debug, warn};

use crate::offer::{MarketOffer, MarketOfferCancelled, MarketOfferId, MarketOffersTable};

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketRateLimiter {
    by_actor: HashMap<u32, VecDeque<SystemTime>>,
}

impl MarketRateLimiter {
    pub(crate) fn record_offer_create(
        &mut self,
        actor_id: u32,
        now: SystemTime,
        minute_limit: usize,
        hour_limit: usize,
    ) -> bool {
        let entries = self.by_actor.entry(actor_id).or_default();
        prune_entries(entries, now, Duration::from_secs(60 * 60));

        let within_hour = entries.len();
        let within_minute = entries
            .iter()
            .filter(|timestamp| {
                now.duration_since(**timestamp)
                    .map(|elapsed| elapsed <= Duration::from_secs(60))
                    .unwrap_or(false)
            })
            .count();

        if within_minute >= minute_limit || within_hour >= hour_limit {
            return false;
        }

        entries.push_back(now);
        true
    }
}

#[derive(Debug, Resource, Default)]
pub(crate) struct MarketOfferSequence {
    next_counter: Wrapping<u16>,
}

impl MarketOfferSequence {
    pub(crate) fn seed_from_offers(&mut self, offers: &MarketOffersTable) {
        self.next_counter = Wrapping(
            offers
                .iter()
                .map(|offer| offer.id().counter())
                .max()
                .unwrap_or(0),
        );
    }

    pub(crate) fn next(&mut self) -> u16 {
        self.next_counter += Wrapping(1);
        self.next_counter.0
    }
}

pub(crate) fn cancel_offer(event: &MarketOfferCancelled, offers: &mut MarketOffersTable) {
    if event.offer().is_none() {
        debug!(
            "Skipping market offer cancellation for {:?}: offer {:?} was not cached",
            event.entity(),
            event.offer_id()
        );
        return;
    }

    let _ = offers.remove(&event.offer_id());
}

pub(crate) fn accept_offer(
    offer_id: MarketOfferId,
    accepted_amount: u16,
    previous_offer: Option<MarketOffer>,
    offers: &mut MarketOffersTable,
) -> Option<MarketOffer> {
    let Some(existing_offer) = previous_offer else {
        debug!(
            "Skipping market offer accept for {:?}: offer {:?} was not cached",
            offer_id, accepted_amount
        );
        return None;
    };

    if accepted_amount == 0 {
        warn!(
            "Ignoring market offer accept for {:?}: accepted amount was zero",
            offer_id
        );
        return Some(existing_offer);
    }

    if accepted_amount >= existing_offer.amount() {
        let _ = offers.remove(&offer_id);
        return None;
    }

    let updated_offer = MarketOffer::new(
        existing_offer.id(),
        existing_offer.item_id(),
        existing_offer.actor_id(),
        existing_offer.amount() - accepted_amount,
        existing_offer.price(),
        existing_offer.side(),
        existing_offer.is_anonymous(),
    );

    offers.insert(updated_offer.clone());
    Some(updated_offer)
}

fn prune_entries(entries: &mut VecDeque<SystemTime>, now: SystemTime, window: Duration) {
    while let Some(front) = entries.front().copied() {
        let expired = now
            .duration_since(front)
            .map(|elapsed| elapsed > window)
            .unwrap_or(false);

        if expired {
            entries.pop_front();
        } else {
            break;
        }
    }
}
