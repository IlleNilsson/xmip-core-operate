//! The surface over a node's operate listener, followed over gRPC
//! (ADR-0014, amendments 2026-10-12 and 2026-10-13): on another machine at
//! its operate address inside Xmip's mutual TLS (`REMOTE`), or on this one
//! at its local endpoint, the operating system proving who this side runs
//! as (`LOCAL`). The node reached is `runtime::plane::Remote`; where it
//! gathers its cluster's view, that view is what this follows (ADR-0052,
//! amendment 2026-11-12), as the runtime's follow says.

mod follow;
mod secret;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use observe::topology::from_to::FromTo;
use observe::{Act, Count, Counted, DeadMessage, EventSubscription, HealthRecord, Noun};
use observe::{Subscription, Unheard};
use runtime::plane::Remote;
use runtime::plane::remote::{Clustered, Follows};
use xcore::Clock;
use xcore::lock::Locked;

use crate::surface::acted::{ActOutcome, Acted, ScopeAction, ScopeOperation};
use crate::surface::acted::{SecretOperation, SecretRecord};
use crate::surface::failed_message_list::FailedMessageList;
use crate::surface::figures::Figures;
use crate::surface::publication_reading::PublicationReading;
use crate::surface::run_header::RunHeader;
use crate::surface::scope_index::ScopeIndex;
use crate::surface::{OperatorSurface, SurfaceChange};

/// How long a question waits for the follow's first telling, which says
/// what the node's answer decided; the runtime tells at once whether the
/// node answered, so this bounds only a node that does neither.
const DECIDING: Duration = Duration::from_secs(30);

/// What a local surface naming no endpoint says on every answer.
pub const UNNAMED: &str = "no node is named: give -Local or --local the node's local endpoint, \
    xmip-<node> unless its [operate] local names another, or name a surface in the document";

/// Why a line naming `--remote` with no address cannot be obeyed.
#[must_use]
pub fn remote_refusal(address: Option<&str>) -> Option<&'static str> {
    address
        .is_none_or(|address| address.trim().is_empty())
        .then_some("A remote needs a node's operate address, like node1:7401.")
}

/// Why a line naming `--local` with no endpoint cannot be obeyed.
#[must_use]
pub fn local_refusal(endpoint: Option<&str>) -> Option<&'static str> {
    endpoint
        .is_none_or(|endpoint| endpoint.trim().is_empty())
        .then_some("A local node needs its local endpoint, like xmip-node1.")
}

/// What the follow knows.
struct State {
    connected: bool,
    reason: String,
    held: Option<(u64, Arc<PublicationReading>)>,
    told_revision: u64,
    follows: Follows,
    gathered: String,
}

/// A node reached at its operate listener, or why it is not.
pub struct ListenerSurface {
    kind: &'static str,
    address: String,
    node: Option<Remote>,
    state: Mutex<State>,
    reading: Mutex<()>,
    following: AtomicBool,
}

impl ListenerSurface {
    /// The node at `address`, presenting `identity` — or, where it is no
    /// identity, why.
    #[must_use]
    pub fn remote(
        address: &str,
        identity: Result<tls::Identity, String>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let node = identity.and_then(|identity| Remote::open(address, &identity, clock));
        Self::over("REMOTE", address, node)
    }

    /// The node on this machine at its local endpoint `endpoint`; none
    /// named is [`UNNAMED`].
    #[must_use]
    pub fn local(endpoint: Option<&str>, clock: Arc<dyn Clock>) -> Self {
        Self::local_within(endpoint, Duration::ZERO, clock)
    }

    /// The same, each call waiting at most `within` — zero for what the node
    /// declared (`runtime::plane::Remote::local`).
    #[must_use]
    pub fn local_within(endpoint: Option<&str>, within: Duration, clock: Arc<dyn Clock>) -> Self {
        let named = endpoint.filter(|endpoint| !endpoint.trim().is_empty());
        let node = named.map_or_else(
            || Err(UNNAMED.to_string()),
            |endpoint| Remote::local(endpoint, within, clock),
        );
        Self::over("LOCAL", endpoint.unwrap_or_default(), node)
    }

    fn over(kind: &'static str, address: &str, node: Result<Remote, String>) -> Self {
        let (node, reason) = match node {
            Ok(node) => (Some(node), "not connected yet".to_string()),
            Err(reason) => (None, reason),
        };

        Self {
            kind,
            address: address.to_string(),
            node,
            state: Mutex::new(State {
                connected: false,
                reason,
                held: None,
                told_revision: 0,
                follows: Follows::Answered,
                gathered: String::new(),
            }),
            reading: Mutex::new(()),
            following: AtomicBool::new(false),
        }
    }

    /// Whether the node answers now.
    pub fn connect(&self) -> bool {
        let _ = self.read();
        self.state.locked().connected
    }

    /// Whether the node answered when last asked, why not where it did
    /// not, and whether what it follows is the cluster's view its node
    /// gathers — nothing asked of the node.
    #[must_use]
    pub fn standing(&self) -> (bool, String, bool) {
        let state = self.state.locked();
        (
            state.connected,
            state.reason.clone(),
            state.follows == Follows::Cluster,
        )
    }

    /// What it answers from now: what it follows, read once per revision
    /// the follow told; nothing where no node answers.
    #[must_use]
    pub fn reading(&self) -> Arc<PublicationReading> {
        self.read()
    }

    // What this follows, read once per revision the follow told; read on
    // every question where nothing follows. Unreachable is nothing, never
    // what was held from before.
    fn read(&self) -> Arc<PublicationReading> {
        let Some(node) = &self.node else {
            self.state.locked().connected = false;
            return Arc::new(PublicationReading::nothing(&self.source()));
        };
        let _one = self.reading.locked();

        {
            let state = self.state.locked();
            if self.following.load(Ordering::Acquire)
                && state.connected
                && let Some((_, held)) = &state.held
            {
                return Arc::clone(held);
            }
        }

        let (revision, text) = match self.fetch(node) {
            Ok(read) => read,
            Err(reason) => {
                let mut state = self.state.locked();
                state.connected = false;
                state.reason = reason;
                drop(state);
                return Arc::new(PublicationReading::nothing(&self.source()));
            }
        };

        {
            let mut state = self.state.locked();
            state.connected = true;
            state.reason = String::new();
            if let Some((held_revision, held)) = &state.held
                && *held_revision == revision
            {
                return Arc::clone(held);
            }
        }

        let read = Arc::new(PublicationReading::of(&text, revision, &self.source()));
        let mut state = self.state.locked();
        state.gathered = if state.follows == Follows::Cluster {
            read.said.clone()
        } else {
            String::new()
        };
        // A reading older than what the follow has since told is answered,
        // not kept: the next question reads the newer.
        if revision >= state.told_revision {
            state.held = Some((revision, Arc::clone(&read)));
        }

        read
    }

    // What the follow says this follows, read: where it has said nothing
    // yet, its telling is waited for first.
    fn fetch(&self, node: &Remote) -> Result<(u64, String), String> {
        if self.state.locked().follows == Follows::Answered {
            let told = node.follow(Follows::Answered, 0, DECIDING)?;
            self.state.locked().follows = told.follows;
            if told.follows == Follows::Answered {
                return Err(told.reason);
            }
        }

        if self.state.locked().follows == Follows::Cluster {
            match node.cluster()? {
                Clustered::Gathered(revision, text) => Ok((revision, text)),
                Clustered::GathersNone(said) => Err(said),
            }
        } else {
            node.publication()
        }
    }

    /// `act` on the `noun` called `target` of the node at `at`, as the
    /// caller this process holds by `caller` (`runtime::plane::held`), 0
    /// for this side itself: what came of it. A node that did not answer is
    /// lost, and the act refused in the surface's words.
    pub fn act_as(&self, caller: u64, at: &str, noun: Noun, target: &str, act: Act) -> Acted {
        let Some(node) = &self.node else {
            return Acted::declined(self.source());
        };

        match node.act(caller, at, (noun.word(), target, act.word())) {
            Ok((outcome, said)) => Acted {
                outcome: ActOutcome::of(outcome),
                said,
            },
            Err(said) => {
                self.lost(said);
                Acted::declined(self.source())
            }
        }
    }

    /// `act` on the failed Message of `journey` at `scope`, asked of its
    /// node as `caller` (0: this side): the node and what came of it.
    pub fn act_message_as(
        &self,
        caller: u64,
        scope: &str,
        journey: &str,
        act: Act,
    ) -> (String, Acted) {
        crate::surface::failed_message_list::asked_on_node(scope, journey, act, |node| {
            self.act_as(caller, node, Noun::Message, journey, act)
        })
    }

    fn lost(&self, reason: String) {
        let mut state = self.state.locked();
        state.connected = false;
        state.reason = reason;
    }
}

impl OperatorSurface for ListenerSurface {
    fn source(&self) -> String {
        let state = self.state.locked();

        if !state.connected {
            format!(
                "{} — {} unreachable: {}",
                self.kind, self.address, state.reason
            )
        } else if !state.gathered.is_empty() {
            format!("{} — {} — {}", self.kind, self.address, state.gathered)
        } else {
            format!("{} — {}", self.kind, self.address)
        }
    }

    fn health(&self, scope: &str) -> Vec<HealthRecord> {
        self.read().index.health(scope)
    }

    fn measure(&self, scope: &str, counted: Counted) -> Option<Count> {
        let now = i64::try_from(xcore::SystemClock.unix_timestamp_nanos()).unwrap_or(i64::MAX);
        self.read().index.measure(scope, counted, now)
    }

    fn pause_scope(&self, scope: &str, who: &str) -> String {
        self.control(scope, ScopeAction::Pause, who).result
    }

    fn resume_scope(&self, scope: &str) -> String {
        self.control(scope, ScopeAction::Resume, "").result
    }

    fn figures(&self, scope: &str) -> Figures {
        self.read().index.figures(scope)
    }

    fn root(&self) -> String {
        self.read().root.clone()
    }

    fn index(&self) -> Arc<ScopeIndex> {
        Arc::clone(&self.read().index)
    }

    fn paths(&self) -> Arc<HashMap<String, FromTo>> {
        self.read().paths()
    }

    fn run(&self) -> RunHeader {
        self.read().run.clone()
    }

    fn watch(&self, stop: &AtomicBool, each: &mut dyn FnMut(&SurfaceChange)) {
        self.follow(stop, each);
    }

    fn subscriptions(&self) -> Vec<Subscription> {
        self.read().subscriptions.clone()
    }

    fn event_subscriptions(&self) -> (Vec<EventSubscription>, Vec<Unheard>) {
        let read = self.read();
        (read.event_subscriptions.clone(), read.unheard.clone())
    }

    fn dead_messages(&self) -> Vec<DeadMessage> {
        self.read().dead_messages.clone()
    }

    fn failed_messages(&self, scope: &str, from: u64, most: u32) -> FailedMessageList {
        self.read().failed_messages.within(scope).paged(from, most)
    }

    fn act(&self, node: &str, noun: Noun, target: &str, act: Act, _who: &str) -> Acted {
        self.act_as(0, node, noun, target, act)
    }

    fn act_message(&self, scope: &str, journey: &str, act: Act, _who: &str) -> (String, Acted) {
        self.act_message_as(0, scope, journey, act)
    }

    fn control(&self, scope: &str, action: ScopeAction, _who: &str) -> ScopeOperation {
        ScopeOperation {
            scope: scope.to_string(),
            action,
            applied: false,
            result: format!(
                "{} takes no pause or resume of a scope over its operate listener",
                self.address
            ),
        }
    }

    fn secrets(&self, node: &str) -> Result<Vec<SecretRecord>, String> {
        self.secrets_as(0, node)
    }

    fn set_secret(
        &self,
        node: &str,
        name: &str,
        given: (&str, &str),
        _who: &str,
    ) -> SecretOperation {
        self.set_secret_as(0, node, name, given)
    }
}
