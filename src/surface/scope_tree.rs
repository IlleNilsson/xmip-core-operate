//! The scope tree, once (ADR-0027 clause 3): what is beneath what, how a
//! parent rolls up, and which record stands worst. Each rule is
//! `node::Scope`'s, `node::Stage`'s, `observe::Health`'s or
//! `observe::Standing`'s, called here; what is composed of them — a parent,
//! a node's own scope, a branch of the drill — is written here for every
//! surface.

use node::{Scope, Stage};
use observe::{Counted, Health, HealthRecord, Standing};

/// The root of the tree, which contains every scope.
pub const ROOT: &str = "xmip:///";

/// The segments of `scope`, top first; none for the root.
#[must_use]
pub fn parts(scope: &str) -> Vec<&str> {
    Scope::new(scope).segments().collect()
}

/// The scope `parts` name.
#[must_use]
pub fn join<S: AsRef<str>>(parts: &[S]) -> String {
    let mut joined = String::from(ROOT);

    for (at, part) in parts.iter().enumerate() {
        if at > 0 {
            joined.push('/');
        }
        joined.push_str(part.as_ref());
    }

    joined
}

/// Whether `candidate` is `scope` or beneath it.
#[must_use]
pub fn beneath(candidate: &str, scope: &str) -> bool {
    Scope::new(scope).contains(Scope::new(candidate))
}

/// The scope one step up; the root's parent is the root.
#[must_use]
pub fn parent(scope: &str) -> String {
    let parts = parts(scope);

    if parts.len() <= 1 {
        ROOT.to_string()
    } else {
        join(&parts[..parts.len() - 1])
    }
}

/// The node `scope` is on, by position; empty on none.
#[must_use]
pub fn node(scope: &str) -> &str {
    Scope::new(scope).node().unwrap_or_default()
}

/// The node's own scope that `scope` is on or beneath; empty on no node.
#[must_use]
pub fn node_scope(scope: &str) -> String {
    let on = node(scope);

    if on.is_empty() {
        return String::new();
    }

    let mut at = scope.to_string();

    loop {
        let up = parent(&at);

        if up == at || node(&up) != on {
            return at;
        }
        at = up;
    }
}

/// The stage `scope` is on, its word; empty on none.
#[must_use]
pub fn stage(scope: &str) -> &'static str {
    Scope::new(scope).stage().map_or("", Stage::name)
}

/// What the stage `stage` names counts, or `None` for a word that is no
/// stage.
#[must_use]
pub fn counted_at(stage: &str) -> Option<Counted> {
    Stage::named(stage).map(Counted::at)
}

/// Whether `scope` is on a stage an operator may pause.
#[must_use]
pub fn pausable(scope: &str) -> bool {
    Stage::named(stage(scope)).is_some_and(Stage::pausable)
}

/// The segment at `index`; empty past the last.
#[must_use]
pub fn segment(scope: &str, index: usize) -> &str {
    parts(scope).get(index).copied().unwrap_or_default()
}

/// What a scope is called beneath its cluster: its path after the cluster,
/// or the scope itself where it has no more than the cluster.
#[must_use]
pub fn name(scope: &str) -> String {
    let parts = parts(scope);

    if parts.len() >= 2 {
        parts[1..].join("/")
    } else {
        scope.to_string()
    }
}

/// Worst first, as the runtime orders records (`observe::Standing`).
#[must_use]
pub fn worst_first(records: &[HealthRecord]) -> Vec<HealthRecord> {
    let order = Standing::worst_first(
        &records
            .iter()
            .map(HealthRecord::standing)
            .collect::<Vec<_>>(),
    );

    order.into_iter().map(|at| records[at].clone()).collect()
}

/// `items` worst first, each standing as the record `stands_as` makes of
/// it.
#[must_use]
pub fn worst_first_by<T: Clone>(items: &[T], stands_as: impl Fn(&T) -> HealthRecord) -> Vec<T> {
    let standing: Vec<HealthRecord> = items.iter().map(stands_as).collect();
    let order = Standing::worst_first(
        &standing
            .iter()
            .map(HealthRecord::standing)
            .collect::<Vec<_>>(),
    );

    order.into_iter().map(|at| items[at].clone()).collect()
}

/// The worst of `records`, or `None` for none.
#[must_use]
pub fn worst(records: &[HealthRecord]) -> Option<HealthRecord> {
    let standing: Vec<_> = records.iter().map(HealthRecord::standing).collect();

    Standing::worst_first(&standing)
        .first()
        .map(|&at| records[at].clone())
}

/// What a parent whose worst leaf is `worst` rolls up to.
#[must_use]
pub fn rolled(worst: Health) -> Health {
    worst.rolled()
}

/// The rollup of `records`, or `None` for none.
#[must_use]
pub fn rollup(records: &[HealthRecord]) -> Option<Health> {
    worst(records).map(|record| rolled(record.health))
}

/// A record standing for an item in the worst-first order: its scope, mood
/// and severity, and nothing else.
#[must_use]
pub fn standing(scope: &str, health: Health, severity: u8) -> HealthRecord {
    HealthRecord {
        scope: scope.to_string(),
        health,
        severity,
        evidence: String::new(),
        observed_unix_nanos: 0,
    }
}

/// One thing directly beneath a scope in the drill: its scope, the segment
/// it is called by, its mood — its own as a leaf, the rollup as a parent —
/// whether it is a leaf, how many leaves it holds, and its worst leaf.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    pub scope: String,
    pub label: String,
    pub health: Health,
    pub is_leaf: bool,
    pub count: usize,
    pub worst: HealthRecord,
}

impl Branch {
    /// How the branch stands in the worst-first order: its worst leaf's mood
    /// and severity, told apart by its label.
    #[must_use]
    pub fn standing(&self) -> HealthRecord {
        standing(&self.label, self.worst.health, self.worst.severity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaf(scope: &str, health: Health, severity: u8) -> HealthRecord {
        HealthRecord {
            scope: scope.to_string(),
            health,
            severity,
            evidence: String::new(),
            observed_unix_nanos: 0,
        }
    }

    #[test]
    fn a_scope_is_its_segments_and_its_parent_is_one_step_up() {
        let test = configure::fixture::test_cluster();
        let (cluster, node) = (test.scope(), test.node(0).name.clone());
        let at = test.node_scope(0);

        assert!(parts(ROOT).is_empty());
        assert_eq!(
            parts(&format!("{at}/receive")),
            [test.name.as_str(), "node", node.as_str(), "receive"]
        );
        assert_eq!(
            join(&[test.name.as_str(), "node"]),
            format!("{cluster}/node")
        );
        assert_eq!(parent(&at), format!("{cluster}/node"));
        assert_eq!(parent(&cluster), ROOT);
        assert_eq!(parent(ROOT), ROOT);
        assert_eq!(name(&at), format!("node/{node}"));
        assert_eq!(name(&cluster), cluster);
    }

    #[test]
    fn a_node_s_own_scope_is_found_from_beneath_it() {
        let test = configure::fixture::test_cluster();
        let (cluster, at) = (test.scope(), test.node_scope(0));

        assert_eq!(node(&format!("{at}/send/out")), test.node(0).name);
        assert_eq!(node_scope(&format!("{at}/send/out")), at);
        assert_eq!(node_scope(&format!("{cluster}/send")), "");
        assert_eq!(stage(&format!("{at}/send/out")), "send");
        assert_eq!(stage(&at), "");
    }

    #[test]
    fn the_worst_stands_first_and_a_parent_rolls_it_up() {
        let test = configure::fixture::test_cluster();
        let (fine, exhausted) = (test.node_scope(0), test.node_scope(1));
        let records = [
            leaf(&fine, Health::Fine, 0),
            leaf(&exhausted, Health::Exhausted, 40),
        ];

        assert_eq!(
            worst(&records).map(|record| record.scope),
            Some(exhausted.clone())
        );
        assert_eq!(rollup(&records), Some(Health::Exhausted.rolled()));
        assert_eq!(worst_first(&records)[0].scope, exhausted);
        assert_eq!(rollup(&[]), None);
    }
}
