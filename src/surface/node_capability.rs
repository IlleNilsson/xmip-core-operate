//! What one node declares (ADR-0056): its roles, its Node capabilities —
//! `online` among them where a route off its machine may be assumed
//! (amendment 2026-11-24) — and whether the node published it or the run
//! started it with it. The reading of a declaration is `node::Capability`'s
//! and `observe::capability`'s; this is what a surface says of one.

use node::{Capability, NodeRole, Stage};

/// What a surface says of a node declaring no Node capability.
const NO_NODE_CAPABILITY: &str = "no Node capability";

/// One node's declaration, or [`NodeCapability::none`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NodeCapability {
    pub node: String,
    pub roles: Vec<String>,
    /// Its Node capabilities, in the order declared.
    pub capabilities: Vec<String>,
    /// Published by the node; otherwise what the run started it with.
    pub published: bool,
    pub evidence: String,
    /// The REFUSED sentence where the declaration names no role.
    pub refusal: String,
}

impl NodeCapability {
    /// Nothing declared.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// Whether a node said anything.
    #[must_use]
    pub fn said(&self) -> bool {
        !self.node.is_empty()
    }

    /// The roles as one word, `receiving+sending`.
    #[must_use]
    pub fn words(&self) -> String {
        self.roles.join("+")
    }

    /// The stages its roles serve, in the message path's order.
    #[must_use]
    pub fn stages(&self) -> Vec<&'static str> {
        Stage::ALL
            .into_iter()
            .filter(|stage| {
                self.roles.iter().any(|role| {
                    NodeRole::named(role).is_some_and(|role| role.stages().contains(stage))
                })
            })
            .map(Stage::name)
            .collect()
    }

    /// Its Node capabilities in one phrase, `online, erp-direct`, or `no
    /// Node capability`.
    #[must_use]
    pub fn named(&self) -> String {
        if self.capabilities.is_empty() {
            NO_NODE_CAPABILITY.to_string()
        } else {
            self.capabilities.join(", ")
        }
    }

    /// Where the declaration came from, in words.
    #[must_use]
    pub fn origin(&self) -> &'static str {
        if self.published {
            "published by the node"
        } else {
            "what the run started it with"
        }
    }

    /// One line: its roles and Node capabilities, or its refusal; empty for
    /// none.
    #[must_use]
    pub fn line(&self) -> String {
        if !self.said() {
            String::new()
        } else if !self.refusal.is_empty() {
            self.refusal.clone()
        } else if self.roles.is_empty() {
            format!("no role · {}", self.named())
        } else {
            format!("{} · {}", self.words(), self.named())
        }
    }

    /// What the node is for, each role it declares with its meaning, or its
    /// refusal; empty for none (ADR-0056, amendment 2026-11-13).
    #[must_use]
    pub fn purpose(&self) -> String {
        if !self.said() {
            String::new()
        } else if !self.refusal.is_empty() {
            self.refusal.clone()
        } else {
            purpose(&self.roles)
        }
    }

    /// The declaration a capability record at `scope` publishes, or `None`
    /// for a record of another kind.
    #[must_use]
    pub fn declared(scope: &str, evidence: &str) -> Option<Self> {
        let (node, said) = observe::capability::declared(scope, evidence)?;
        let mut declared = Self::from(node, said);
        declared.published = true;
        declared.evidence = evidence.to_string();

        Some(declared)
    }

    /// What a run started a node with, from its `[run]` entry,
    /// `<node>=receiving+sending`; its Node capabilities are the run's list
    /// to say (`RunHeader::capability`).
    #[must_use]
    pub fn started(entry: &str) -> Self {
        let (node, said) = Capability::from_entry(entry);
        Self::from(node, said)
    }

    fn from(node: &str, said: Result<Capability, String>) -> Self {
        match said {
            Ok(capability) => Self {
                node: node.to_string(),
                roles: capability
                    .roles()
                    .iter()
                    .map(|role| role.name().to_string())
                    .collect(),
                capabilities: capability.declared().to_vec(),
                ..Self::default()
            },
            Err(refusal) => Self {
                node: node.to_string(),
                refusal,
                ..Self::default()
            },
        }
    }
}

/// What a node declaring `roles` is for, each role with its meaning
/// (`node::NodeRole::purpose`), or the refusal of a word that is no role.
#[must_use]
pub fn purpose<S: AsRef<str>>(roles: &[S]) -> String {
    let declared: Vec<&str> = roles.iter().map(AsRef::as_ref).collect();

    match NodeRole::declared(&declared.join(",")) {
        Ok(roles) => NodeRole::purpose(&roles),
        Err(refusal) => refusal,
    }
}
