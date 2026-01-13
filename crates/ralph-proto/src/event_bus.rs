//! Event bus for pub/sub messaging.
//!
//! The event bus routes events to subscribed hats based on topic patterns.

use crate::{Event, Hat, HatId};
use std::collections::HashMap;

/// Central pub/sub hub for routing events between hats.
#[derive(Debug, Default)]
pub struct EventBus {
    /// Registered hats indexed by ID.
    hats: HashMap<HatId, Hat>,

    /// Pending events for each hat.
    pending: HashMap<HatId, Vec<Event>>,
}

impl EventBus {
    /// Creates a new empty event bus.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a hat with the event bus.
    pub fn register(&mut self, hat: Hat) {
        let id = hat.id.clone();
        self.hats.insert(id.clone(), hat);
        self.pending.entry(id).or_default();
    }

    /// Publishes an event to all subscribed hats.
    ///
    /// Returns the list of hat IDs that received the event.
    pub fn publish(&mut self, event: Event) -> Vec<HatId> {
        let mut recipients = Vec::new();

        // If there's a direct target, route only to that hat
        if let Some(ref target) = event.target {
            if self.hats.contains_key(target) {
                self.pending
                    .entry(target.clone())
                    .or_default()
                    .push(event.clone());
                recipients.push(target.clone());
            }
            return recipients;
        }

        // Otherwise, route to all subscribers
        for (id, hat) in &self.hats {
            // Don't route back to source
            if event.source.as_ref() == Some(id) {
                continue;
            }

            if hat.is_subscribed(&event.topic) {
                self.pending
                    .entry(id.clone())
                    .or_default()
                    .push(event.clone());
                recipients.push(id.clone());
            }
        }

        recipients
    }

    /// Takes all pending events for a hat.
    pub fn take_pending(&mut self, hat_id: &HatId) -> Vec<Event> {
        self.pending.remove(hat_id).unwrap_or_default()
    }

    /// Checks if there are any pending events for any hat.
    pub fn has_pending(&self) -> bool {
        self.pending.values().any(|events| !events.is_empty())
    }

    /// Returns the next hat with pending events.
    pub fn next_hat_with_pending(&self) -> Option<&HatId> {
        self.pending
            .iter()
            .find(|(_, events)| !events.is_empty())
            .map(|(id, _)| id)
    }

    /// Gets a hat by ID.
    pub fn get_hat(&self, id: &HatId) -> Option<&Hat> {
        self.hats.get(id)
    }

    /// Returns all registered hat IDs.
    pub fn hat_ids(&self) -> impl Iterator<Item = &HatId> {
        self.hats.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_to_subscriber() {
        let mut bus = EventBus::new();

        let hat = Hat::new("impl", "Implementer").subscribe("task.*");
        bus.register(hat);

        let event = Event::new("task.start", "Start implementing");
        let recipients = bus.publish(event);

        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].as_str(), "impl");
    }

    #[test]
    fn test_no_match() {
        let mut bus = EventBus::new();

        let hat = Hat::new("impl", "Implementer").subscribe("task.*");
        bus.register(hat);

        let event = Event::new("review.done", "Review complete");
        let recipients = bus.publish(event);

        assert!(recipients.is_empty());
    }

    #[test]
    fn test_direct_target() {
        let mut bus = EventBus::new();

        let impl_hat = Hat::new("impl", "Implementer").subscribe("task.*");
        let review_hat = Hat::new("reviewer", "Reviewer").subscribe("impl.*");
        bus.register(impl_hat);
        bus.register(review_hat);

        // Direct target bypasses subscription matching
        let event = Event::new("handoff", "Please review").with_target("reviewer");
        let recipients = bus.publish(event);

        assert_eq!(recipients.len(), 1);
        assert_eq!(recipients[0].as_str(), "reviewer");
    }

    #[test]
    fn test_take_pending() {
        let mut bus = EventBus::new();

        let hat = Hat::new("impl", "Implementer").subscribe("*");
        bus.register(hat);

        bus.publish(Event::new("task.start", "Start"));
        bus.publish(Event::new("task.continue", "Continue"));

        let hat_id = HatId::new("impl");
        let events = bus.take_pending(&hat_id);

        assert_eq!(events.len(), 2);
        assert!(bus.take_pending(&hat_id).is_empty());
    }

    #[test]
    fn test_no_self_routing() {
        let mut bus = EventBus::new();

        let hat = Hat::new("impl", "Implementer").subscribe("*");
        bus.register(hat);

        let event = Event::new("impl.done", "Done").with_source("impl");
        let recipients = bus.publish(event);

        // Event should not route back to source
        assert!(recipients.is_empty());
    }
}
