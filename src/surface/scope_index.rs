//! One publication as the scope tree it is (ADR-0027): every scope with its
//! leaves worst first, its worst, its own record, its rollup, its children
//! worst first and its six figures, all computed in one pass when the
//! publication is read, then answered by lookup. A scope holds its records
//! as their places in the publication, never copies, so a surface across the
//! C ABI is answered by place and keeps the records it read
//! (`ffi::surface::index`).

mod lookup;

pub use lookup::Placed;

use std::collections::{BTreeMap, HashMap};

use observe::{Count, Health, HealthRecord, Standing};

use crate::surface::figures::KINDS;
use crate::surface::node_capability::NodeCapability;
use crate::surface::scope_tree::{self, ROOT};

/// One scope and what the index knows of it.
#[derive(Clone, Debug)]
struct Entry {
    scope: String,
    label: String,
    parent: Option<usize>,
    stage: &'static str,
    begins_stage: bool,
    /// The records at and beneath it, worst first, by place.
    leaves: Vec<u32>,
    children: Vec<usize>,
    /// The record published at it, the last there, by place.
    own: Option<u32>,
    /// How many records are published at it itself, not beneath it.
    owned: u32,
    sums: [Option<u64>; 6],
}

/// A publication's scope tree.
#[derive(Clone, Debug)]
pub struct ScopeIndex {
    records: Vec<HealthRecord>,
    entries: Vec<Entry>,
    at: HashMap<String, usize>,
    stages: HashMap<&'static str, u32>,
    declared: BTreeMap<String, NodeCapability>,
    /// The publication's revision.
    pub revision: u64,
    /// Who published it.
    pub source: String,
    /// When its newest figure was observed.
    pub observed: Option<i64>,
}

impl ScopeIndex {
    /// Nothing published, said by `source`.
    #[must_use]
    pub fn empty(source: &str) -> Self {
        Self::build(Vec::new(), &[], 0, source)
    }

    /// The tree `records` and `counts` make; each record keeps its place.
    #[must_use]
    pub fn build(
        records: Vec<HealthRecord>,
        counts: &[Count],
        revision: u64,
        source: &str,
    ) -> Self {
        let mut index = Self {
            records: Vec::new(),
            entries: vec![Entry {
                scope: ROOT.to_string(),
                label: "cluster".to_string(),
                parent: None,
                stage: "",
                begins_stage: false,
                leaves: Vec::new(),
                children: Vec::new(),
                own: None,
                owned: 0,
                sums: [None; 6],
            }],
            at: HashMap::from([(ROOT.to_string(), 0)]),
            stages: HashMap::new(),
            declared: BTreeMap::new(),
            revision,
            source: source.to_string(),
            observed: None,
        };

        for record in &records {
            if let Some(said) = NodeCapability::declared(&record.scope, &record.evidence) {
                index.declared.insert(said.node.clone(), said);
            }
        }

        let standing: Vec<_> = records.iter().map(HealthRecord::standing).collect();
        let order = Standing::worst_first(&standing);
        drop(standing);

        for at in order {
            index.hold(&records[at].scope, place(at));
        }
        index.records = records;

        for count in counts {
            index.count(count);
        }

        index.order();
        index
    }

    fn hold(&mut self, scope: &str, at: u32) {
        self.entries[0].leaves.push(at);
        let mut entry = 0;

        for reached in self.ancestry(scope) {
            self.entries[reached].leaves.push(at);
            entry = reached;
        }

        let owner = &mut self.entries[entry];
        owner.owned += 1;
        if owner.own.is_none_or(|before| at > before) {
            owner.own = Some(at);
        }

        let stage = owner.stage;
        if !stage.is_empty() {
            self.stages.entry(stage).or_insert(at);
        }
    }

    fn count(&mut self, count: &Count) {
        let Some(kind) = KINDS.iter().position(|&kind| kind == count.counted) else {
            return;
        };
        let add = |sum: &mut Option<u64>| *sum = Some(sum.unwrap_or(0).wrapping_add(count.value));

        add(&mut self.entries[0].sums[kind]);
        for reached in self.ancestry(&count.scope) {
            add(&mut self.entries[reached].sums[kind]);
        }

        self.observed = self.observed.max(Some(count.observed_unix_nanos));
    }

    // The entry of each scope from the cluster down to `scope`, made where
    // none is.
    fn ancestry(&mut self, scope: &str) -> Vec<usize> {
        let parts = scope_tree::parts(scope);
        let mut reached = Vec::with_capacity(parts.len());
        let mut above = 0;

        for depth in 0..parts.len() {
            let joined = scope_tree::join(&parts[..=depth]);
            let entry = if let Some(&entry) = self.at.get(&joined) {
                entry
            } else {
                let stage = scope_tree::stage(&joined);
                let entry = self.entries.len();
                self.entries.push(Entry {
                    scope: joined.clone(),
                    label: parts[depth].to_string(),
                    parent: Some(above),
                    stage,
                    begins_stage: !stage.is_empty() && self.entries[above].stage.is_empty(),
                    leaves: Vec::new(),
                    children: Vec::new(),
                    own: None,
                    owned: 0,
                    sums: [None; 6],
                });
                self.entries[above].children.push(entry);
                self.at.insert(joined, entry);
                entry
            };
            reached.push(entry);
            above = entry;
        }

        reached
    }

    // Every parent's children worst first, told apart by label.
    fn order(&mut self) {
        for parent in 0..self.entries.len() {
            if self.entries[parent].children.len() < 2 {
                continue;
            }
            let children = self.entries[parent].children.clone();
            let ordered =
                scope_tree::worst_first_by(&children, |&child| self.standing(&self.entries[child]));
            self.entries[parent].children = ordered;
        }
    }

    fn worst_of(&self, entry: &Entry) -> Option<&HealthRecord> {
        entry.leaves.first().map(|&at| &self.records[at as usize])
    }

    fn mood_of(&self, entry: &Entry) -> Option<Health> {
        let worst = self.worst_of(entry)?.health;
        Some(if entry.children.is_empty() {
            worst
        } else {
            scope_tree::rolled(worst)
        })
    }

    fn standing(&self, entry: &Entry) -> HealthRecord {
        let (health, severity) = self
            .worst_of(entry)
            .map_or((Health::Fine, 0), |worst| (worst.health, worst.severity));
        scope_tree::standing(&entry.label, health, severity)
    }

    fn entry(&self, scope: &str) -> Option<&Entry> {
        if scope == ROOT {
            return self.entries.first();
        }
        let normal = scope_tree::join(&scope_tree::parts(scope));
        self.at.get(&normal).map(|&at| &self.entries[at])
    }
}

// A record's place, as the C ABI carries it.
fn place(at: usize) -> u32 {
    u32::try_from(at).unwrap_or(u32::MAX)
}
