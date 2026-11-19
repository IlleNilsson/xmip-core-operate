//! A publication as a surface answers from it: the text a node published —
//! read from its file (`snapshot`) or over the node's operate listener
//! (`listener`) — read once by the one reader, `observe::Publication`, and
//! laid out as the tree and the lists every command reads. Written once for
//! both. The publication read is kept whole, so a surface in another
//! language is handed it as section 8 lends one (`ffi::surface`) and reads
//! nothing a second time.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use observe::topology::from_to::{FromTo, from_to};
use observe::{DeadMessage, EventSubscription, Publication, Subscription, Unheard};

use crate::surface::failed_message_list::FailedMessageList;
use crate::surface::run_header::RunHeader;
use crate::surface::scope_index::ScopeIndex;
use crate::surface::scope_tree::ROOT;

/// One publication, read.
#[derive(Clone, Debug)]
pub struct PublicationReading {
    /// The publication as it was read; empty where nothing was published.
    pub publication: Arc<Publication>,
    pub index: Arc<ScopeIndex>,
    pub run: RunHeader,
    /// Where a drill starts.
    pub root: String,
    pub subscriptions: Vec<Subscription>,
    pub event_subscriptions: Vec<EventSubscription>,
    pub unheard: Vec<Unheard>,
    pub dead_messages: Vec<DeadMessage>,
    pub failed_messages: FailedMessageList,
    /// Who published, in the publication's own words: a gathered view's
    /// `<cluster>, gathered by <node>` among them; empty where it names no
    /// one.
    pub said: String,
    paths: OnceLock<Arc<HashMap<String, FromTo>>>,
}

impl PublicationReading {
    /// Nothing published, said by `source`.
    #[must_use]
    pub fn nothing(source: &str) -> Self {
        Self {
            publication: Arc::default(),
            index: Arc::new(ScopeIndex::empty(source)),
            run: RunHeader::default(),
            root: ROOT.to_string(),
            subscriptions: Vec::new(),
            event_subscriptions: Vec::new(),
            unheard: Vec::new(),
            dead_messages: Vec::new(),
            failed_messages: FailedMessageList::unlisted(),
            said: String::new(),
            paths: OnceLock::new(),
        }
    }

    /// `text` read as revision `revision`; its own source where it names
    /// one, `source` otherwise. Text the reader refuses is nothing
    /// published.
    #[must_use]
    pub fn of(text: &str, revision: u64, source: &str) -> Self {
        let Ok(read) = Publication::read(text) else {
            return Self::nothing(source);
        };

        let said = if read.source.is_empty() {
            source
        } else {
            read.source.as_str()
        };
        // Each count keeps when it was observed, so a figure is as old as
        // its publication says (ADR-0027 clause 6).
        let index = ScopeIndex::build(read.records.clone(), &read.counts, revision, said);
        // Where a drill starts: the document's node — a roll's cluster —
        // where a record is published beneath it; the root otherwise.
        let published_at_node = index
            .placed(&read.node)
            .is_some_and(|node| !node.leaves.is_empty());
        let root = if !read.node.is_empty() && published_at_node {
            read.node.clone()
        } else {
            ROOT.to_string()
        };

        Self {
            index: Arc::new(index),
            run: RunHeader::from(read.run.as_ref()),
            root,
            subscriptions: read.subscriptions.clone(),
            event_subscriptions: read.event_subscriptions.clone(),
            unheard: read.unheard.clone(),
            dead_messages: read.dead_messages.clone(),
            failed_messages: FailedMessageList::published(&read.failed_messages),
            said: read.source.clone(),
            paths: OnceLock::new(),
            publication: Arc::new(read),
        }
    }

    /// The from and to of everything the publication draws, by the scope it
    /// is drawn at, worked out the first time it is asked.
    #[must_use]
    pub fn paths(&self) -> Arc<HashMap<String, FromTo>> {
        Arc::clone(
            self.paths
                .get_or_init(|| Arc::new(paths(&self.publication))),
        )
    }
}

// The from and to of everything the publication draws, by the scope it is
// drawn at; a scope beneath one of them has its from and to.
fn paths(read: &Publication) -> HashMap<String, FromTo> {
    let Some(topology) = read.topology.as_ref() else {
        return HashMap::new();
    };
    let snapshot = read.snapshot();
    let mut paths = HashMap::new();

    for node in &topology.nodes {
        if node.scope.is_empty() || paths.contains_key(&node.scope) {
            continue;
        }
        let said = from_to(&snapshot, Some(topology), &node.scope);
        if !said.hops.is_empty() {
            paths.insert(node.scope.clone(), said);
        }
    }

    paths
}
