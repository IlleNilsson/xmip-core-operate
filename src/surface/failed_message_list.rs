//! The Messages that failed at a node's Send Ports, as a surface lists
//! them (runtime-model.md section 13; ADR-0013): Port by Port, how many
//! wait, and a page of them — read from Xmip Storage where the node runs in
//! the process, or the window its publication carries. A list says which of
//! three it is: listed, not listed — whether any wait is not known — or not
//! answered, with why.

use observe::{Act, FailedMessages, Noun};

use crate::surface::OperatorSurface;
use crate::surface::acted::Acted;
use crate::surface::scope_tree;

/// One Port's failed Messages: how many wait, a page of them, and where the
/// next page starts in the Port's index of failed Messages — `None` where
/// the list was read from a publication, which carries no place in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedPort {
    pub node: String,
    pub send_port: String,
    pub count: u64,
    pub next: Option<u64>,
    pub messages: Vec<observe::FailedMessage>,
    /// Whether the oldest blocks its sequence.
    pub blocked: bool,
}

/// The Messages that failed, Port by Port, or why they were not listed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailedMessageList {
    pub ports: Vec<FailedPort>,
    /// Whether the surface could list them at all.
    pub listed: bool,
    /// `FAILED:` and why, where the surface asked and was not answered.
    pub failure: String,
}

impl FailedMessageList {
    /// Listed: these Ports.
    #[must_use]
    pub fn listed(ports: Vec<FailedPort>) -> Self {
        Self {
            ports,
            listed: true,
            failure: String::new(),
        }
    }

    /// Not listed: whether any wait is not known.
    #[must_use]
    pub fn unlisted() -> Self {
        Self {
            ports: Vec::new(),
            listed: false,
            failure: String::new(),
        }
    }

    /// Asked and not answered, for `why`.
    #[must_use]
    pub fn unanswered(why: &str) -> Self {
        Self {
            ports: Vec::new(),
            listed: false,
            failure: format!("FAILED: {why}"),
        }
    }

    /// What a publication carries, its node's window of each Port.
    #[must_use]
    pub fn published(ports: &[FailedMessages]) -> Self {
        Self::listed(
            ports
                .iter()
                .map(|port| FailedPort {
                    node: port.node.clone(),
                    send_port: port.send_port.clone(),
                    count: port.count,
                    next: None,
                    messages: port.messages.clone(),
                    blocked: port.blocked,
                })
                .collect(),
        )
    }

    /// The Ports at or beneath `scope`: every Port of the nodes beneath a
    /// scope on no node, a node's own Ports, or one Port where `scope` is
    /// on its node's stage beneath it ([`port_at`]).
    #[must_use]
    pub fn within(&self, scope: &str) -> Self {
        let (node, port) = port_at(scope);

        Self {
            ports: self
                .ports
                .iter()
                .filter(|at| {
                    (if node.is_empty() {
                        scope_tree::beneath(&at.node, scope)
                    } else {
                        at.node == node
                    }) && (port.is_empty() || at.send_port == port)
                })
                .cloned()
                .collect(),
            listed: self.listed,
            failure: self.failure.clone(),
        }
    }

    /// What a list that is not one says, `NOT LISTED:` or its `FAILED:`;
    /// empty for a list.
    #[must_use]
    pub fn unlisted_said(&self, scope: &str, source: &str) -> String {
        if !self.failure.is_empty() {
            self.failure.clone()
        } else if self.listed {
            String::new()
        } else {
            format!(
                "NOT LISTED: {source} cannot list the Messages that failed, so whether any wait \
                 at or beneath {scope} is not known — not that none do."
            )
        }
    }

    /// Each Port's Messages from the place `from` in its queue on, at most
    /// `most` of each (0: every one).
    #[must_use]
    pub fn paged(&self, from: u64, most: u32) -> Self {
        let take = if most == 0 {
            usize::MAX
        } else {
            usize::try_from(most).unwrap_or(usize::MAX)
        };

        Self {
            ports: self
                .ports
                .iter()
                .map(|port| FailedPort {
                    messages: port
                        .messages
                        .iter()
                        .filter(|message| message.sequence >= from)
                        .take(take)
                        .cloned()
                        .collect(),
                    ..port.clone()
                })
                .collect(),
            listed: self.listed,
            failure: self.failure.clone(),
        }
    }
}

/// The node's own scope `scope` is on, and the Send Port where `scope` is
/// on a stage beneath it — the segment after the stage's — else empty.
#[must_use]
pub fn port_at(scope: &str) -> (String, String) {
    let node = scope_tree::node_scope(scope);

    if node.is_empty() || scope_tree::stage(scope).is_empty() {
        return (node, String::new());
    }

    let depth = scope_tree::parts(&node).len();
    let port = scope_tree::segment(scope, depth + 1).to_string();

    (node, port)
}

/// The node an act on the failed Message of `journey` at `scope` is asked
/// of: the node's own scope `scope` is on.
///
/// # Errors
///
/// No Journey named, or a scope on no node: the REFUSED sentence.
pub fn on_node(scope: &str, journey: &str, act: &str) -> Result<String, String> {
    if journey.is_empty() {
        return Err(format!("REFUSED: to {act} a Message, name its Journey"));
    }

    let node = scope_tree::node_scope(scope);

    if node.is_empty() {
        Err(format!(
            "REFUSED: {scope} is on no node; name the node that sends the Message's Send Port, \
             or the Send Port's scope beneath it"
        ))
    } else {
        Ok(node)
    }
}

/// An act on the failed Message of `journey` at `scope`, asked of the node
/// `scope` is on ([`on_node`]) through `surface`, or refused before any
/// node is asked: the node it names, and what came of it.
pub fn acted_on_node(
    surface: &(impl OperatorSurface + ?Sized),
    scope: &str,
    journey: &str,
    (act, who): (Act, &str),
) -> (String, Acted) {
    asked_on_node(scope, journey, act, |node| {
        surface.act(node, Noun::Message, journey, act, who)
    })
}

/// The same, `ask` asking the node it names: for a surface that asks as a
/// caller of its own (`ffi::surface`).
pub fn asked_on_node(
    scope: &str,
    journey: &str,
    act: Act,
    ask: impl FnOnce(&str) -> Acted,
) -> (String, Acted) {
    match on_node(scope, journey, act.word()) {
        Ok(node) => {
            let acted = ask(&node);
            (node, acted)
        }
        Err(refused) => (scope.to_string(), Acted::declined(refused)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn port(node: &str, send_port: &str) -> FailedPort {
        FailedPort {
            node: node.to_string(),
            send_port: send_port.to_string(),
            count: 1,
            next: None,
            messages: Vec::new(),
            blocked: false,
        }
    }

    #[test]
    fn a_list_narrows_to_a_cluster_a_node_or_one_port() {
        let test = configure::fixture::test_cluster();
        let (cluster, a, b) = (test.scope(), test.node_scope(0), test.node_scope(1));
        let list = FailedMessageList::listed(vec![port(&a, "p"), port(&a, "q"), port(&b, "p")]);

        assert_eq!(list.within(&cluster).ports.len(), 3);
        assert_eq!(list.within(&a).ports.len(), 2);
        assert_eq!(list.within(&format!("{a}/send/q")).ports.len(), 1);
        assert_eq!(port_at(&format!("{a}/send/q/x")), (a.clone(), "q".into()));
        assert!(on_node(&cluster, "j", "retry").is_err());
        assert!(on_node(&a, "", "retry").is_err());
        assert!(
            FailedMessageList::unlisted()
                .unlisted_said("s", "src")
                .starts_with("NOT LISTED")
        );
    }
}
