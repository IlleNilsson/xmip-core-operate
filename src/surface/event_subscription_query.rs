//! What a surface asks of the Event subscriptions it lists (ADR-0065,
//! amendment 2026-09-29), in the words every surface uses: the pattern,
//! `*` and `?` over each one's node and the scope it reaches; where,
//! `--location`; which, `--id`; the column and order of the list; and which
//! members the nodes there do not hear (ADR-0065, amendment 2026-10-02).

use observe::{EventSubscription, PauseState, Unheard};

use crate::surface::list_order::{Key, ListOrder, cluster, patterned, placed};
use crate::surface::scope_tree;

/// The columns an Event subscription is sorted by, the first the default.
pub const COLUMNS: [&str; 9] = [
    "subscriber",
    "cluster",
    "node",
    "action",
    "state",
    "queued",
    "delivered",
    "missed",
    "since",
];

/// What a line asks of the Event subscriptions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventSubscriptionQuery {
    pub pattern: Option<String>,
    pub location: Option<String>,
    pub id: Option<u64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl EventSubscriptionQuery {
    /// The Event subscriptions of `subscriptions` the query selects, in its
    /// order.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn apply(
        &self,
        subscriptions: &[EventSubscription],
    ) -> Result<Vec<EventSubscription>, String> {
        let places = self.places(subscriptions)?;
        Ok(places
            .into_iter()
            .map(|at| subscriptions[at].clone())
            .collect())
    }

    /// The places in `subscriptions` of those the query selects, in its
    /// order.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn places(&self, subscriptions: &[EventSubscription]) -> Result<Vec<usize>, String> {
        let order = ListOrder::asked(self.sort.as_deref(), self.order.as_deref(), &COLUMNS)?;

        Ok(order.places(
            subscriptions,
            |subscription| self.selected(subscription),
            key,
            |entry| (entry.node.clone(), entry.id),
        ))
    }

    /// The members the nodes at or above `--location` do not hear — every
    /// one where none is named — by who does not hear and whom.
    #[must_use]
    pub fn unheard(&self, unheard: &[Unheard]) -> Vec<Unheard> {
        self.unheard_places(unheard)
            .into_iter()
            .map(|at| unheard[at].clone())
            .collect()
    }

    /// The places in `unheard` of those [`Self::unheard`] keeps, in its
    /// order.
    #[must_use]
    pub fn unheard_places(&self, unheard: &[Unheard]) -> Vec<usize> {
        let location = self
            .location
            .as_deref()
            .filter(|location| !location.is_empty());
        let mut places: Vec<usize> = (0..unheard.len())
            .filter(|&at| {
                let gone = &unheard[at];
                location.is_none_or(|location| {
                    scope_tree::beneath(&gone.by, location)
                        || scope_tree::beneath(location, &gone.by)
                })
            })
            .collect();

        places.sort_by(|&a, &b| {
            let (a, b) = (&unheard[a], &unheard[b]);
            a.by.cmp(&b.by).then_with(|| a.node.cmp(&b.node))
        });
        places
    }

    fn selected(&self, subscription: &EventSubscription) -> bool {
        let numbered = self.id.is_none_or(|id| subscription.id == id);

        placed(&subscription.node, self.location.as_deref())
            && numbered
            && patterned(
                &[&subscription.node, &subscription.scope],
                self.pattern.as_deref(),
            )
    }
}

/// Who a subscriber is: the name it declared, its Party's identifier where
/// it declared none.
#[must_use]
pub fn who(subscription: &EventSubscription) -> &str {
    if subscription.subscriber.is_empty() {
        &subscription.party
    } else {
        &subscription.subscriber
    }
}

/// The cluster an Event subscription's node is in.
#[must_use]
pub fn cluster_of(subscription: &EventSubscription) -> String {
    cluster(&subscription.node)
}

/// The name of the node an Event subscription is held on.
#[must_use]
pub fn node_name(subscription: &EventSubscription) -> String {
    scope_tree::node(&subscription.node).to_string()
}

/// Whether an operator paused it.
#[must_use]
pub fn paused(subscription: &EventSubscription) -> bool {
    subscription.state == PauseState::Paused
}

/// One member not heard, as a person reads it: the node that does not hear
/// it, then what it says.
#[must_use]
pub fn line(unheard: &Unheard) -> String {
    said_by(&unheard.by, &unheard.said())
}

/// The line of a member not heard by `by` that says `said`.
#[must_use]
pub fn said_by(by: &str, said: &str) -> String {
    format!("{}: {said}", scope_tree::node(by))
}

fn key(entry: &EventSubscription, column: &str) -> Key {
    match column {
        "cluster" => Key::Text(cluster_of(entry)),
        "node" => Key::Text(node_name(entry)),
        "action" => Key::Text(entry.action.clone()),
        "state" => Key::Text(entry.state.word().to_string()),
        "queued" => Key::Number(entry.queued),
        "delivered" => Key::Number(entry.delivered),
        "missed" => Key::Number(entry.missed),
        "since" => Key::moment(entry.since_unix_nanos),
        _ => Key::Text(who(entry).to_string()),
    }
}

#[cfg(test)]
mod tests {
    use configure::fixture::test_cluster;

    use super::*;
    use crate::surface::publication_reading::PublicationReading;

    fn entry(cluster: &str, node: &str, id: u32, said: (&str, &str, u32, &str)) -> String {
        let (action, state, queued, scope) = said;
        let named = match id {
            1 => "subscriber = \"operations\"\n",
            2 => "subscriber = \"on-call\"\n",
            _ => "",
        };
        format!(
            "[[event_subscriptions]]\nnode = \"{cluster}/node/{node}\"\nid = {id}\n{named}\
             party = \"0199a0a0-0000-7000-8000-00000000000{id}\"\naction = \"{action}\"\n\
             scope = \"{scope}\"\nstate = \"{state}\"\nqueued = {queued}\n"
        )
    }

    #[test]
    fn the_query_drills_filters_and_sorts() {
        let cluster = test_cluster();
        let scope = cluster.scope();
        let receiver = cluster.with_role("receiving").name.clone();
        let sender = cluster.with_role("sending").name.clone();
        let sending = format!("{scope}/node/{sender}");
        let receiving = format!("{scope}/node/{receiver}");
        let text = format!(
            "node = \"{scope}\"\n{}{}{}",
            entry(
                &scope,
                &receiver,
                1,
                ("every Event", "active", 3, &receiving)
            ),
            entry(
                &scope,
                &sender,
                2,
                ("every Event ending failure", "paused", 9, &sending)
            ),
            entry(
                &scope,
                &sender,
                4,
                ("se.xmip.send.failure", "active", 1, &scope)
            ),
        );
        let all = PublicationReading::of(&text, 1, "test").event_subscriptions;
        let ids = |query: EventSubscriptionQuery| -> Vec<u64> {
            query
                .apply(&all)
                .expect("applied")
                .into_iter()
                .map(|one| one.id)
                .collect()
        };

        let mut by_id = all.clone();
        by_id.sort_by_key(|one| one.id);
        assert_eq!(
            by_id.iter().map(who).collect::<Vec<_>>(),
            [
                "operations",
                "on-call",
                "0199a0a0-0000-7000-8000-000000000004"
            ]
        );
        assert_eq!(ids(EventSubscriptionQuery::default()), [4, 2, 1]);
        assert_eq!(
            ids(EventSubscriptionQuery {
                location: Some(sending.clone()),
                ..EventSubscriptionQuery::default()
            }),
            [4, 2]
        );
        assert_eq!(
            ids(EventSubscriptionQuery {
                location: Some(sending),
                id: Some(4),
                ..EventSubscriptionQuery::default()
            }),
            [4]
        );
        assert_eq!(
            ids(EventSubscriptionQuery {
                sort: Some("queued".into()),
                order: Some("descending".into()),
                ..EventSubscriptionQuery::default()
            }),
            [2, 1, 4]
        );
        assert_eq!(
            ids(EventSubscriptionQuery {
                pattern: Some(format!("*/{receiver}")),
                ..EventSubscriptionQuery::default()
            }),
            [1]
        );
        assert_eq!(cluster_of(&all[0]), cluster.name);
        assert_eq!(node_name(&all[0]), receiver);

        let refused = EventSubscriptionQuery {
            sort: Some("mood".into()),
            ..EventSubscriptionQuery::default()
        }
        .apply(&all);
        assert!(refused.is_err_and(|said| said.starts_with("REFUSED")));
    }
}
