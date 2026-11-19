//! What a scope index answers: by record, for a Rust surface, and by a
//! record's place in the publication, for a surface across the C ABI that
//! keeps the records it read.

use observe::{Count, Counted, Health, HealthRecord};

use super::{Entry, ScopeIndex};
use crate::surface::figures::{Figures, KINDS};
use crate::surface::node_capability::NodeCapability;
use crate::surface::scope_tree::{self, Branch};

/// One scope as the index holds it, every record by its place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Placed<'a> {
    pub scope: &'a str,
    pub label: &'a str,
    /// The records at and beneath it, worst first.
    pub leaves: &'a [u32],
    /// The record published at it.
    pub own: Option<u32>,
    /// Its mood: its own as a leaf, the rollup as a parent.
    pub mood: Option<Health>,
    pub is_leaf: bool,
    /// Whether records lie beneath it, not only at it.
    pub is_container: bool,
    /// Its mood as its row says it: the rollup where records lie beneath
    /// it, else its worst record's own.
    pub row_mood: Option<Health>,
    pub begins_stage: bool,
    pub stage: &'static str,
    pub sums: [Option<u64>; 6],
}

impl ScopeIndex {
    /// Every record the publication holds, in the order it published them.
    #[must_use]
    pub fn records(&self) -> &[HealthRecord] {
        &self.records
    }

    /// `scope` as the index holds it, by place; `None` where the tree has
    /// no such scope.
    #[must_use]
    pub fn placed(&self, scope: &str) -> Option<Placed<'_>> {
        self.entry(scope).map(|entry| self.view(entry))
    }

    /// The scopes directly beneath `scope`, worst first, by place.
    #[must_use]
    pub fn children_placed(&self, scope: &str) -> Vec<Placed<'_>> {
        self.entry(scope).map_or_else(Vec::new, |entry| {
            entry
                .children
                .iter()
                .map(|&child| self.view(&self.entries[child]))
                .collect()
        })
    }

    fn view<'a>(&'a self, entry: &'a Entry) -> Placed<'a> {
        let is_container = entry.leaves.len() > entry.owned as usize;
        let worst = self.worst_of(entry).map(|worst| worst.health);

        Placed {
            scope: &entry.scope,
            label: &entry.label,
            leaves: &entry.leaves,
            own: entry.own,
            mood: self.mood_of(entry),
            is_leaf: entry.children.is_empty(),
            is_container,
            row_mood: if is_container {
                worst.map(scope_tree::rolled)
            } else {
                worst
            },
            begins_stage: entry.begins_stage,
            stage: entry.stage,
            sums: entry.sums,
        }
    }

    /// The place of the worst record on the stage `stage`.
    #[must_use]
    pub fn worst_at_stage_placed(&self, stage: &str) -> Option<u32> {
        self.stages.get(stage).copied()
    }

    /// How many records the publication holds.
    #[must_use]
    pub fn leaves(&self) -> usize {
        self.entries[0].leaves.len()
    }

    /// How many scopes the tree has.
    #[must_use]
    pub fn scopes(&self) -> usize {
        self.entries.len()
    }

    /// What `node` declared, by its last capability record.
    #[must_use]
    pub fn capability(&self, node: &str) -> NodeCapability {
        self.declared.get(node).cloned().unwrap_or_default()
    }

    /// Every node's declaration, by node.
    #[must_use]
    pub fn capabilities(&self) -> Vec<NodeCapability> {
        self.declared.values().cloned().collect()
    }

    /// The scopes `pattern` matches, in ordinal order.
    #[must_use]
    pub fn matching(&self, pattern: &str) -> Vec<String> {
        owned(self.matching_borrowed(pattern))
    }

    /// The scopes `pattern` matches, in ordinal order, borrowed.
    #[must_use]
    pub fn matching_borrowed(&self, pattern: &str) -> Vec<&str> {
        let mut scopes: Vec<&str> = self
            .at
            .keys()
            .filter(|scope| node::wildcard::matches(scope, pattern))
            .map(String::as_str)
            .collect();
        scopes.sort_unstable();
        scopes
    }

    /// The scopes where the stage `stage` begins, in ordinal order,
    /// borrowed.
    #[must_use]
    pub fn stage_scopes_borrowed(&self, stage: &str) -> Vec<&str> {
        let mut scopes: Vec<&str> = self
            .entries
            .iter()
            .filter(|entry| entry.begins_stage && entry.stage == stage)
            .map(|entry| entry.scope.as_str())
            .collect();
        scopes.sort_unstable();
        scopes
    }

    /// The names directly beneath the root, in ordinal order, borrowed.
    #[must_use]
    pub fn nodes_borrowed(&self) -> Vec<&str> {
        let mut nodes: Vec<&str> = self.entries[0]
            .children
            .iter()
            .map(|&child| self.entries[child].label.as_str())
            .collect();
        nodes.sort_unstable();
        nodes
    }

    /// What a person reads `scope` as: a stage by its title, else its
    /// segment — borrowed from the index, the title, or `scope` itself.
    #[must_use]
    pub fn label_borrowed<'a>(&'a self, scope: &'a str) -> &'a str {
        match self.entry(scope) {
            Some(entry) if entry.begins_stage => {
                node::Stage::named(&entry.label).map_or(entry.label.as_str(), |stage| stage.title())
            }
            Some(entry) => &entry.label,
            None => node::Scope::new(scope).segments().last().unwrap_or(scope),
        }
    }

    /// Whether the tree has `scope`.
    #[must_use]
    pub fn contains(&self, scope: &str) -> bool {
        self.entry(scope).is_some()
    }

    /// The records at and beneath `scope`, worst first.
    #[must_use]
    pub fn health(&self, scope: &str) -> Vec<HealthRecord> {
        self.entry(scope).map_or_else(Vec::new, |entry| {
            entry
                .leaves
                .iter()
                .map(|&at| self.records[at as usize].clone())
                .collect()
        })
    }

    /// The record published at `scope` itself, the last one there.
    #[must_use]
    pub fn own(&self, scope: &str) -> Option<HealthRecord> {
        let at = self.entry(scope)?.own?;
        Some(self.records[at as usize].clone())
    }

    /// The worst record at or beneath `scope`.
    #[must_use]
    pub fn worst(&self, scope: &str) -> Option<HealthRecord> {
        self.worst_of(self.entry(scope)?).cloned()
    }

    /// `scope`'s mood: its own as a leaf, the rollup as a parent.
    #[must_use]
    pub fn rollup(&self, scope: &str) -> Option<Health> {
        self.mood_of(self.entry(scope)?)
    }

    /// The six figures summed at `scope`.
    #[must_use]
    pub fn figures(&self, scope: &str) -> Figures {
        self.entry(scope).map_or_else(
            || Figures::none(scope),
            |entry| Figures {
                scope: scope.to_string(),
                values: entry.sums,
                observed: self.observed,
            },
        )
    }

    /// One figure at `scope`, observed when the publication's newest was,
    /// or `now` where it says none.
    #[must_use]
    pub fn measure(&self, scope: &str, counted: Counted, now: i64) -> Option<Count> {
        let kind = KINDS.iter().position(|&kind| kind == counted)?;
        let value = self.entry(scope)?.sums[kind]?;
        let seen = self.observed.unwrap_or(now);

        Some(Count {
            scope: scope.to_string(),
            counted,
            value,
            window_start_unix_nanos: seen.saturating_sub(60_000_000_000),
            window_end_unix_nanos: seen,
            observed_unix_nanos: seen,
        })
    }

    /// The scopes directly beneath `scope`, worst first.
    #[must_use]
    pub fn branches(&self, scope: &str) -> Vec<Branch> {
        self.children_placed(scope)
            .into_iter()
            .map(|child| Branch {
                scope: child.scope.to_string(),
                label: child.label.to_string(),
                health: child.mood.unwrap_or(Health::Fine),
                is_leaf: child.is_leaf,
                count: child.leaves.len(),
                worst: child.leaves.first().map_or_else(
                    || scope_tree::standing(child.scope, Health::Fine, 0),
                    |&at| self.records[at as usize].clone(),
                ),
            })
            .collect()
    }

    /// The scopes where the stage `stage` begins, in ordinal order.
    #[must_use]
    pub fn stage_scopes(&self, stage: &str) -> Vec<String> {
        owned(self.stage_scopes_borrowed(stage))
    }

    /// What a person reads `scope` as: a stage by its title, else its
    /// segment.
    #[must_use]
    pub fn label(&self, scope: &str) -> String {
        self.label_borrowed(scope).to_string()
    }

    /// The worst record on the stage `stage`.
    #[must_use]
    pub fn worst_at_stage(&self, stage: &str) -> Option<HealthRecord> {
        let at = self.worst_at_stage_placed(stage)?;
        Some(self.records[at as usize].clone())
    }

    /// The names directly beneath the root, in ordinal order.
    #[must_use]
    pub fn nodes(&self) -> Vec<String> {
        owned(self.nodes_borrowed())
    }

    /// The scope's parent entry's scope, where the tree has both.
    #[must_use]
    pub fn parent(&self, scope: &str) -> Option<String> {
        let parent = self.entry(scope)?.parent?;
        Some(self.entries[parent].scope.clone())
    }
}

fn owned(borrowed: Vec<&str>) -> Vec<String> {
    borrowed.into_iter().map(str::to_string).collect()
}
