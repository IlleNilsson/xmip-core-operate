//! What a surface asks of the Dead Message Queues it lists (ADR-0052,
//! amendment 2026-10-01), in the words every surface uses: the pattern,
//! `*` and `?` over each entry's node or its scope
//! `<node>/dead-message/<message>`; where, `--location`; which,
//! `--message`; and the column and order of the list, oldest first by
//! default.

use observe::DeadMessage;

use crate::surface::list_order::{Key, ListOrder, cluster, patterned, placed};
use crate::surface::scope_tree;

/// The columns an entry is sorted by, the first the default.
pub const COLUMNS: [&str; 6] = [
    "received",
    "message",
    "cluster",
    "node",
    "receive-location",
    "declines",
];

/// The noun a Message in a Dead Message Queue is, in its scope and its act.
pub const NOUN: &str = "dead-message";

/// What a line asks of the Dead Message Queues.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DeadMessageQuery {
    pub pattern: Option<String>,
    pub location: Option<String>,
    pub message: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

impl DeadMessageQuery {
    /// The entries of `entries` the query selects, in its order.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn apply(&self, entries: &[DeadMessage]) -> Result<Vec<DeadMessage>, String> {
        let places = self.places(entries)?;
        Ok(places.into_iter().map(|at| entries[at].clone()).collect())
    }

    /// The places in `entries` of those the query selects, in its order.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn places(&self, entries: &[DeadMessage]) -> Result<Vec<usize>, String> {
        let order = ListOrder::asked(self.sort.as_deref(), self.order.as_deref(), &COLUMNS)?;

        Ok(order.places(
            entries,
            |entry| self.selected(entry),
            key,
            |entry| (entry.node.clone(), entry.sequence),
        ))
    }

    fn selected(&self, entry: &DeadMessage) -> bool {
        let named = self
            .message
            .as_deref()
            .filter(|message| !message.is_empty())
            .is_none_or(|message| entry.message == message);

        placed(&entry.node, self.location.as_deref())
            && named
            && patterned(&[&entry.node, &scope(entry)], self.pattern.as_deref())
    }
}

/// The cluster an entry's node is in.
#[must_use]
pub fn cluster_of(entry: &DeadMessage) -> String {
    cluster(&entry.node)
}

/// The name of the node whose queue keeps an entry.
#[must_use]
pub fn node_name(entry: &DeadMessage) -> String {
    scope_tree::node(&entry.node).to_string()
}

/// An entry's own scope, `<node>/dead-message/<message>`.
#[must_use]
pub fn scope(entry: &DeadMessage) -> String {
    format!("{}/{NOUN}/{}", entry.node, entry.message)
}

fn key(entry: &DeadMessage, column: &str) -> Key {
    match column {
        "message" => Key::Text(entry.message.clone()),
        "cluster" => Key::Text(cluster_of(entry)),
        "node" => Key::Text(node_name(entry)),
        "receive-location" => Key::Text(entry.location.clone()),
        "declines" => Key::Number(u64::try_from(entry.declines.len()).unwrap_or(u64::MAX)),
        _ => Key::moment(entry.received_unix_nanos),
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
        (message, sequence, millis): (&str, u32, u64),
        rest: &str,
    ) -> String {
        format!(
            "[[dead_messages]]\nnode = \"{cluster}/node/{node}\"\nmessage = \"{message}\"\n\
             sequence = {sequence}\nlocation = \"orders\"\nreceived_unix_nanos = {}\n{rest}",
            millis * 1_000_000
        )
    }

    #[test]
    fn the_query_drills_filters_and_sorts() {
        let cluster = test_cluster();
        let scope = cluster.scope();
        let receiver = cluster.with_role("receiving").name.clone();
        let processor = cluster.with_role("processing").name.clone();
        let receiving = format!("{scope}/node/{receiver}");
        let text = format!(
            "node = \"{scope}\"\n{}{}{}",
            entry(
                &scope,
                &receiver,
                ("m-1", 1, 1_000),
                "validation = [[\"schema\", \"valid\"]]\n\
                 declines = [[\"structured\", \"no MessageType\"]]\n"
            ),
            entry(
                &scope,
                &receiver,
                ("m-2", 2, 2_000),
                "validation = [[\"schema\", \"valid\"], [\"signature\", \"valid\"]]\n\
                 promoted = [[\"MessageType\", \"Invoice\"]]\n\
                 declines = [[\"structured\", \"MessageType is Invoice, not json\"], \
                 [\"edi\", \"paused\"]]\n"
            ),
            entry(&scope, &processor, ("m-3", 1, 3_000), ""),
        );
        let all = PublicationReading::of(&text, 1, "test").dead_messages;
        let ids = |query: DeadMessageQuery| -> Vec<String> {
            query
                .apply(&all)
                .expect("applied")
                .into_iter()
                .map(|one| one.message)
                .collect()
        };

        assert_eq!(ids(DeadMessageQuery::default()), ["m-1", "m-2", "m-3"]);
        assert_eq!(
            ids(DeadMessageQuery {
                location: Some(receiving.clone()),
                ..DeadMessageQuery::default()
            }),
            ["m-1", "m-2"]
        );
        assert_eq!(
            ids(DeadMessageQuery {
                location: Some(receiving),
                message: Some("m-2".into()),
                ..DeadMessageQuery::default()
            }),
            ["m-2"]
        );
        assert_eq!(
            ids(DeadMessageQuery {
                sort: Some("declines".into()),
                order: Some("descending".into()),
                ..DeadMessageQuery::default()
            }),
            ["m-2", "m-1", "m-3"]
        );
        assert_eq!(
            ids(DeadMessageQuery {
                pattern: Some(format!("*/{processor}")),
                ..DeadMessageQuery::default()
            }),
            ["m-3"]
        );
        assert_eq!(
            ids(DeadMessageQuery {
                pattern: Some("*/dead-message/m-1".into()),
                ..DeadMessageQuery::default()
            }),
            ["m-1"]
        );
        assert_eq!(cluster_of(&all[0]), cluster.name);
        assert_eq!(node_name(&all[0]), receiver);

        let refused = DeadMessageQuery {
            sort: Some("mood".into()),
            ..DeadMessageQuery::default()
        }
        .apply(&all);
        assert!(refused.is_err_and(|said| said.starts_with("REFUSED")));
    }
}
