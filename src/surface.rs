//! What every screen, command and cmdlet reads, implemented once in Rust
//! (ADR-0052 clause 1; ADR-0014, amendment 2026-11-17): the
//! [`OperatorSurface`] a surface answers through, over a published snapshot
//! (`snapshot`) or a node's operate listener, on this machine or another
//! (`listener`), chosen by what the line and the host's document state
//! (`choice`); the scope tree a publication is (`scope_index`), its rows
//! (`scope_item`), what an argument selects (`scope_selection`), the six
//! figures (`figures`), and how each is said to a person (`english`).
//!
//! The rules beneath are the crates': a scope's segments and the wildcard
//! are `node`'s, a mood, its rollup and the worst-first order `observe`'s,
//! a publication's reading `observe::Publication`'s and a node reached at
//! its listener `runtime::plane::Remote`'s.

pub mod acted;
pub mod choice;
pub mod cluster_named;
pub mod configuration_verdict;
pub mod dead_message_query;
pub mod english;
pub mod event_subscription_query;
pub mod failed_message_list;
pub mod figures;
pub mod instance_query;
pub mod list_order;
pub mod listener;
pub mod node_capability;
pub mod publication_reading;
pub mod run_header;
pub mod scope_index;
pub mod scope_item;
pub mod scope_selection;
pub mod scope_tree;
pub mod snapshot;
mod snapshot_watch;
pub mod subscription_query;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use observe::topology::from_to::FromTo;
use observe::{Act, Count, Counted, DeadMessage, EventSubscription, HealthRecord, Naming, Noun};
use observe::{Subscription, Unheard, WorkProcessInstances};

use acted::{Acted, ScopeAction, ScopeOperation, SecretOperation, SecretRecord};
use failed_message_list::FailedMessageList;
use figures::Figures;
use node_capability::NodeCapability;
use run_header::RunHeader;
use scope_index::ScopeIndex;
use scope_item::ScopeItem;
use scope_tree::ROOT;

/// That a surface's publication may have advanced: its revision, and its
/// source when it was told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurfaceChange {
    pub revision: u64,
    pub source: String,
}

/// What a surface reads and the two acts on a scope it takes. Every other
/// answer is composed of these where the surface has nothing better; a
/// surface over a publication answers each from its reading.
pub trait OperatorSurface {
    /// Where its answers come from, in words: `SNAPSHOT — <path>`,
    /// `REMOTE — <address>`, or why it reaches nothing.
    fn source(&self) -> String;

    /// The records at and beneath `scope`, worst first.
    fn health(&self, scope: &str) -> Vec<HealthRecord>;

    /// One figure at `scope`.
    fn measure(&self, scope: &str, counted: Counted) -> Option<Count>;

    /// Pause everything at and beneath `scope`, as `who`: the sentence.
    fn pause_scope(&self, scope: &str, who: &str) -> String;

    /// Resume everything at and beneath `scope`: the sentence.
    fn resume_scope(&self, scope: &str) -> String;

    /// The six figures at `scope`.
    fn figures(&self, scope: &str) -> Figures {
        Figures::read(self, scope)
    }

    /// Where a drill starts: the cluster the surface publishes at.
    fn root(&self) -> String {
        ROOT.to_string()
    }

    /// The whole publication as the scope tree it is.
    fn index(&self) -> Arc<ScopeIndex> {
        Arc::new(ScopeIndex::build(self.health(ROOT), &[], 0, &self.source()))
    }

    /// `scope`'s row.
    fn describe(&self, scope: &str) -> ScopeItem {
        ScopeItem::read(self, scope)
    }

    /// The rows directly beneath `scope`, worst first.
    fn children(&self, scope: &str) -> Vec<ScopeItem> {
        ScopeItem::children(self, scope)
    }

    /// Each thing the publication draws, by its scope, with where its work
    /// came from and went (`observe::topology::from_to`).
    fn paths(&self) -> Arc<HashMap<String, FromTo>> {
        Arc::default()
    }

    /// Where the work observed at `scope` came from and went, its two ends:
    /// as the nearest thing at or above it that the publication draws says.
    fn ends(&self, scope: &str) -> FromTo {
        let paths = self.paths();

        if paths.is_empty() {
            return FromTo::default();
        }

        let mut at = scope_tree::join(&scope_tree::parts(scope));

        loop {
            if let Some(said) = paths.get(&at) {
                return said.clone();
            }
            if at == ROOT {
                return FromTo::default();
            }
            at = scope_tree::parent(&at);
        }
    }

    /// What the run was started with, where the publisher says.
    fn run(&self) -> RunHeader {
        RunHeader::default()
    }

    /// How the cluster is named for a person, where the publisher says: the
    /// short name its file declares and the file (`cluster_named`).
    fn naming(&self) -> Naming {
        Naming::default()
    }

    /// What `node` declares: what it published, else what the run started
    /// it with.
    fn capability(&self, node: &str) -> NodeCapability {
        let published = self.index().capability(node);

        if published.said() {
            published
        } else {
            self.run().capability(node)
        }
    }

    /// Tell `each` that the publication may have advanced — first at once,
    /// then each time it does — until `stop` is set.
    fn watch(&self, stop: &AtomicBool, each: &mut dyn FnMut(&SurfaceChange)) {
        each(&SurfaceChange {
            revision: 0,
            source: self.source(),
        });

        while !stop.load(Ordering::Acquire) {
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// The Subscriptions the nodes route by.
    fn subscriptions(&self) -> Vec<Subscription> {
        Vec::new()
    }

    /// The Event subscriptions the nodes hold, and the members they do not
    /// hear.
    fn event_subscriptions(&self) -> (Vec<EventSubscription>, Vec<Unheard>) {
        (Vec::new(), Vec::new())
    }

    /// What the nodes' Dead Message Queues keep.
    fn dead_messages(&self) -> Vec<DeadMessage> {
        Vec::new()
    }

    /// The Instances of each Work Process the nodes run, and the Messages
    /// that failed in its queue (`runtime-model.md` section 22; open problem
    /// 192).
    fn work_process_instances(&self) -> Vec<WorkProcessInstances> {
        Vec::new()
    }

    /// The Messages that failed at or beneath `scope`, each Port's page
    /// from `from`, at most `most` of each (0: a hundred).
    fn failed_messages(&self, _scope: &str, _from: u64, _most: u32) -> FailedMessageList {
        FailedMessageList::unlisted()
    }

    /// `act` on the `noun` called `target` of the node at `node`, as `who`.
    fn act(&self, _node: &str, noun: Noun, _target: &str, _act: Act, _who: &str) -> Acted {
        declined(&self.source(), noun)
    }

    /// `act` on the failed Message of `journey` at `scope` — the node that
    /// sends its Send Port, or the Port's scope beneath it — as `who`: the
    /// node it was asked of and what came of it.
    fn act_message(&self, scope: &str, journey: &str, act: Act, who: &str) -> (String, Acted) {
        (
            scope.to_string(),
            self.act(scope, Noun::Message, journey, act, who),
        )
    }

    /// Pause or resume `scope`, as `who`.
    fn control(&self, scope: &str, action: ScopeAction, who: &str) -> ScopeOperation {
        let result = match action {
            ScopeAction::Pause => self.pause_scope(scope, who),
            ScopeAction::Resume => self.resume_scope(scope),
        };

        ScopeOperation {
            scope: scope.to_string(),
            action,
            applied: true,
            result,
        }
    }

    /// The secrets the node at `node` keeps, never a value.
    ///
    /// # Errors
    ///
    /// Why they were not listed.
    fn secrets(&self, _node: &str) -> Result<Vec<SecretRecord>, String> {
        Err(format!(
            "{} reaches no node, so lists no secrets",
            self.source()
        ))
    }

    /// Set the secret `name` of `kind` on the node at `node`, as `who`.
    fn set_secret(
        &self,
        node: &str,
        name: &str,
        (_kind, _value): (&str, &str),
        _who: &str,
    ) -> SecretOperation {
        SecretOperation {
            node: node.to_string(),
            name: name.to_string(),
            applied: false,
            result: format!("{} reaches no node to set a secret", self.source()),
        }
    }
}

/// What a surface that acts on no `noun` says to an act on one, naming
/// itself by `source`.
#[must_use]
pub fn declined(source: &str, noun: Noun) -> Acted {
    Acted::declined(match noun {
        Noun::EventSubscription => format!("{source} does not act on Event subscriptions"),
        Noun::Subscription => format!("{source} does not act on Subscriptions"),
        Noun::DeadMessage => format!("{source} does not replay from a Dead Message Queue"),
        Noun::Message => format!("{source} does not act on failed Messages"),
    })
}
