//! One row of the scope tree as every surface shows it: its name, its mood,
//! its worst leaf — scope, severity and evidence, so a Holding row says why
//! and where (ADR-0052 clause 2) — its figures, where its work came from
//! and went, and for a node's own row what the node is for.

use observe::Health;
use observe::topology::from_to::FromTo;

use crate::surface::OperatorSurface;
use crate::surface::figures::Figures;
use crate::surface::scope_index::ScopeIndex;
use crate::surface::scope_selection::ScopeSelection;
use crate::surface::scope_tree::{self, ROOT};

/// One row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeItem {
    pub name: String,
    pub scope: String,
    /// Whether records lie beneath it, not only at it.
    pub is_container: bool,
    /// Its mood; `None` where nothing is recorded.
    pub health: Option<Health>,
    pub severity: Option<u8>,
    pub evidence: String,
    /// Its worst leaf's scope.
    pub worst: Option<String>,
    pub figures: Figures,
    /// When it was last observed, Unix nanoseconds.
    pub observed: Option<i64>,
    pub from_to: Option<FromTo>,
    /// What the node is for, at a node's own scope; empty elsewhere.
    pub purpose: String,
}

impl ScopeItem {
    /// `scope`'s row as `index` holds it, with its `figures`: its worst
    /// record's severity and evidence, and its mood the rollup where records
    /// lie beneath it, else that record's own.
    #[must_use]
    pub fn of(index: &ScopeIndex, scope: &str, figures: Figures) -> Self {
        let placed = index.placed(scope);
        let worst = placed
            .and_then(|placed| placed.leaves.first())
            .map(|&at| &index.records()[at as usize]);
        let observed = figures
            .observed
            .or_else(|| worst.map(|worst| worst.observed_unix_nanos));

        Self {
            name: if scope == ROOT {
                "cluster".to_string()
            } else {
                scope_tree::name(scope)
            },
            scope: scope.to_string(),
            is_container: placed.is_some_and(|placed| placed.is_container),
            health: placed.and_then(|placed| placed.row_mood),
            severity: worst.map(|worst| worst.severity),
            evidence: worst
                .map(|worst| worst.evidence.clone())
                .unwrap_or_default(),
            worst: worst.map(|worst| worst.scope.clone()),
            figures,
            observed,
            from_to: None,
            purpose: String::new(),
        }
    }

    /// `scope`'s row as `surface` answers it.
    #[must_use]
    pub fn read(surface: &(impl OperatorSurface + ?Sized), scope: &str) -> Self {
        let mut item = Self::of(&surface.index(), scope, surface.figures(scope));
        item.from_to = Some(surface.ends(scope));
        item.purpose = purpose_at(surface, scope);
        item
    }

    /// The rows directly beneath `scope`, worst first.
    #[must_use]
    pub fn children(surface: &(impl OperatorSurface + ?Sized), scope: &str) -> Vec<Self> {
        let index = surface.index();

        index
            .branches(scope)
            .into_iter()
            .map(|branch| {
                let at = branch.scope;
                let mut item = Self::of(&index, &at, surface.figures(&at));
                item.from_to = Some(surface.ends(&at));
                item.purpose = purpose_at(surface, &at);
                item
            })
            .collect()
    }

    /// The rows a selection names that exist; under a wildcard, worst first.
    #[must_use]
    pub fn selected(
        surface: &(impl OperatorSurface + ?Sized),
        chosen: &ScopeSelection,
    ) -> Vec<Self> {
        let rows: Vec<Self> = chosen
            .scopes
            .iter()
            .map(|scope| surface.describe(scope))
            .filter(Self::exists)
            .collect();

        if !chosen.patterned || rows.len() < 2 {
            return rows;
        }

        let index = surface.index();
        scope_tree::worst_first_by(&rows, |row| {
            let worst = index.worst(&row.scope);
            scope_tree::standing(
                &row.scope,
                worst.as_ref().map_or(Health::Fine, |worst| worst.health),
                worst.as_ref().map_or(0, |worst| worst.severity),
            )
        })
    }

    /// Where it came from, in words; empty where nothing is drawn.
    #[must_use]
    pub fn came_from(&self) -> &str {
        self.from_to.as_ref().map_or("", |path| path.from.as_str())
    }

    /// Where it went, in words; empty where nothing is drawn.
    #[must_use]
    pub fn went_to(&self) -> &str {
        self.from_to.as_ref().map_or("", |path| path.to.as_str())
    }

    /// Whether its mood is anything but fine.
    #[must_use]
    pub fn troubled(&self) -> bool {
        self.health.is_some_and(|mood| mood != Health::Fine)
    }

    /// Whether anything is recorded or measured at it.
    #[must_use]
    pub fn exists(&self) -> bool {
        self.health.is_some() || self.figures.has_values()
    }
}

// What the node at a node's own scope is for, published or what the run
// started it with; empty for any other scope.
fn purpose_at(surface: &(impl OperatorSurface + ?Sized), scope: &str) -> String {
    let node = scope_tree::node(scope);

    if node.is_empty() || scope_tree::node_scope(scope) != scope {
        String::new()
    } else {
        surface.capability(node).purpose()
    }
}
