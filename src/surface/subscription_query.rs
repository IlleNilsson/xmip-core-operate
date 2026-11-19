//! What a surface asks of the Subscriptions it lists (ADR-0013, amendment
//! 2026-09-30), in the words every surface uses: the pattern, `*` and `?`
//! over each Subscription's node or its scope
//! `<node>/subscription/<name>`; where, `--location`; which, `--name`; and
//! the column and order of the list.

use observe::{PauseState, Subscription};

use crate::surface::list_order::{Key, ListOrder, cluster, patterned, placed};
use crate::surface::scope_tree;

/// The columns a Subscription is sorted by, the first the default.
pub const COLUMNS: [&str; 9] = [
    "subscription",
    "cluster",
    "node",
    "filter",
    "destination",
    "state",
    "picked-up",
    "held",
    "since",
];

/// What a line asks of the Subscriptions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SubscriptionQuery {
    pub pattern: Option<String>,
    pub location: Option<String>,
    pub name: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl SubscriptionQuery {
    /// The Subscriptions of `subscriptions` the query selects, in its order.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn apply(&self, subscriptions: &[Subscription]) -> Result<Vec<Subscription>, String> {
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
    pub fn places(&self, subscriptions: &[Subscription]) -> Result<Vec<usize>, String> {
        let order = ListOrder::asked(self.sort.as_deref(), self.order.as_deref(), &COLUMNS)?;

        Ok(order.places(
            subscriptions,
            |subscription| self.selected(subscription),
            key,
            |entry| (entry.node.clone(), entry.name.clone()),
        ))
    }

    fn selected(&self, subscription: &Subscription) -> bool {
        let named = self
            .name
            .as_deref()
            .filter(|name| !name.is_empty())
            .is_none_or(|name| subscription.name == name);

        placed(&subscription.node, self.location.as_deref())
            && named
            && patterned(
                &[&subscription.node, &scope(subscription)],
                self.pattern.as_deref(),
            )
    }
}

/// The cluster a Subscription's node is in.
#[must_use]
pub fn cluster_of(subscription: &Subscription) -> String {
    cluster(&subscription.node)
}

/// The name of the node a Subscription routes on.
#[must_use]
pub fn node_name(subscription: &Subscription) -> String {
    scope_tree::node(&subscription.node).to_string()
}

/// A Subscription's own scope, `<node>/subscription/<name>`.
#[must_use]
pub fn scope(subscription: &Subscription) -> String {
    format!("{}/subscription/{}", subscription.node, subscription.name)
}

/// Whether an operator paused it.
#[must_use]
pub fn paused(subscription: &Subscription) -> bool {
    subscription.state == PauseState::Paused
}

fn key(entry: &Subscription, column: &str) -> Key {
    match column {
        "cluster" => Key::Text(cluster_of(entry)),
        "node" => Key::Text(node_name(entry)),
        "filter" => Key::Text(entry.filter.clone()),
        "destination" => Key::Text(entry.destination.clone()),
        "state" => Key::Text(entry.state.word().to_string()),
        "picked-up" => Key::Number(entry.picked_up),
        "held" => Key::Number(entry.held),
        "since" => Key::moment(entry.since_unix_nanos),
        _ => Key::Text(entry.name.clone()),
    }
}

#[cfg(test)]
mod tests {
    use configure::fixture::test_cluster;

    use super::*;
    use crate::surface::publication_reading::PublicationReading;

    fn entry(
        cluster: &str,
        node: &str,
        (name, state): (&str, &str),
        counted: (u32, u32),
    ) -> String {
        format!(
            "[[subscriptions]]\nnode = \"{cluster}/node/{node}\"\nname = \"{name}\"\n\
             application = \"RoundTrip\"\nfilter = \"MessageType = '{name}'\"\n\
             destination = \"the Send Port 'RoundTripOut'\"\nstate = \"{state}\"\n\
             picked_up = {}\nheld = {}\n",
            counted.0, counted.1
        )
    }

    #[test]
    fn the_query_drills_filters_and_sorts() {
        let cluster = test_cluster();
        let scope = cluster.scope();
        let processor = cluster.with_role("processing").name.clone();
        let sender = cluster.with_role("sending").name.clone();
        let text = format!(
            "node = \"{scope}\"\n{}{}{}",
            entry(&scope, &processor, ("structured", "active"), (12, 0)),
            entry(&scope, &processor, ("edi", "paused"), (3, 9)),
            entry(&scope, &sender, ("flat", "active"), (40, 0)),
        );
        let all = PublicationReading::of(&text, 1, "test").subscriptions;
        let names = |query: SubscriptionQuery| -> Vec<String> {
            query
                .apply(&all)
                .expect("applied")
                .into_iter()
                .map(|one| one.name)
                .collect()
        };
        let processing = format!("{scope}/node/{processor}");

        assert_eq!(
            names(SubscriptionQuery::default()),
            ["edi", "flat", "structured"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                location: Some(processing.clone()),
                ..SubscriptionQuery::default()
            }),
            ["edi", "structured"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                location: Some(processing),
                name: Some("structured".into()),
                ..SubscriptionQuery::default()
            }),
            ["structured"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                sort: Some("held".into()),
                order: Some("descending".into()),
                ..SubscriptionQuery::default()
            }),
            ["edi", "structured", "flat"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                sort: Some("picked-up".into()),
                order: Some("descending".into()),
                ..SubscriptionQuery::default()
            }),
            ["flat", "structured", "edi"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                pattern: Some(format!("*/{sender}")),
                ..SubscriptionQuery::default()
            }),
            ["flat"]
        );
        assert_eq!(
            names(SubscriptionQuery {
                pattern: Some("*/subscription/ed?".into()),
                ..SubscriptionQuery::default()
            }),
            ["edi"]
        );
        assert_eq!(cluster_of(&all[0]), cluster.name);
        assert_eq!(node_name(&all[0]), processor);

        let refused = SubscriptionQuery {
            sort: Some("mood".into()),
            ..SubscriptionQuery::default()
        }
        .apply(&all);
        assert!(refused.is_err_and(|said| said.starts_with("REFUSED")));
    }
}
