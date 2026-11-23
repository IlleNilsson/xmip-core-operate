//! The surface over a published snapshot: what a node — or the System Tests,
//! after every tick — wrote to a file, read once per publication and
//! answered from that by lookup. A snapshot is a record of what was
//! published and is never acted on.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use observe::topology::from_to::FromTo;
use observe::{Act, Count, Counted, DeadMessage, EventSubscription, HealthRecord, Noun};
use observe::{Subscription, Unheard};
use xcore::Clock;
use xcore::lock::Locked;

use crate::surface::acted::{Acted, ScopeAction, ScopeOperation};
use crate::surface::failed_message_list::FailedMessageList;
use crate::surface::figures::Figures;
use crate::surface::publication_reading::PublicationReading;
use crate::surface::run_header::RunHeader;
use crate::surface::scope_index::ScopeIndex;
use crate::surface::{OperatorSurface, SurfaceChange};

/// Why a snapshot takes no act.
pub const NOT_ACTED: &str = "a snapshot is a record of what was published and is never acted \
    on; ask the node at its operate listener";

/// What was read, and the file's stamp and identity when it was.
struct Cached {
    reading: Option<Arc<PublicationReading>>,
    /// The file's stamp when it was read: when it was written and made,
    /// and its length.
    stamp: Option<(u64, u64, u64)>,
    /// Which file it was, where a follow asked: a publisher replaces the
    /// file whole, and a copy renamed over it may carry every stamp the
    /// last one had (`runtime::directory_watch::identity`). `None` where
    /// only a question read it.
    identity: Option<u128>,
    revision: u64,
}

/// A publication's file, read.
pub struct SnapshotSurface {
    path: PathBuf,
    clock: Arc<dyn Clock>,
    cached: Mutex<Cached>,
    following: AtomicUsize,
}

impl SnapshotSurface {
    /// The surface over the file at `path`, telling the age of what it
    /// answers by `clock`.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>, clock: Arc<dyn Clock>) -> Self {
        Self {
            path: path.into(),
            clock,
            cached: Mutex::new(Cached {
                reading: None,
                stamp: None,
                identity: None,
                revision: 0,
            }),
            following: AtomicUsize::new(0),
        }
    }

    /// The file it reads.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Whether there is a file to read.
    #[must_use]
    pub fn exists(&self) -> bool {
        stamp(&self.path).is_some()
    }

    /// What it answers from now: the reading a follow last told, else the
    /// file as it is, parsed once per publication.
    #[must_use]
    pub fn reading(&self) -> Arc<PublicationReading> {
        self.read()
    }

    fn read(&self) -> Arc<PublicationReading> {
        if self.following.load(Ordering::Acquire) > 0
            && let Some(told) = self.cached.locked().reading.clone()
        {
            return told;
        }
        self.refresh(false).0
    }

    // Parsed once per publication, never per question: the file's write
    // time, when it was made and its length say whether anything changed,
    // the file unopened; told of a change, which file is there says it too,
    // so a copy renamed over the last with every stamp it had is another
    // publication. A file that could not be read at all is a moment, not a
    // verdict — a publisher renaming over it — and what was read last
    // stands (`false`).
    fn refresh(&self, told: bool) -> (Arc<PublicationReading>, bool) {
        let mut cached = self.cached.locked();
        let file = stamp(&self.path);
        let known = match file {
            Some(_) if told => match identity(&self.path) {
                None => return (self.held(&cached), false),
                known => known,
            },
            _ => None,
        };

        if let Some(held) = &cached.reading
            && cached.stamp == file
            && (known.is_none() || cached.identity.is_none() || cached.identity == known)
        {
            let held = Arc::clone(held);
            cached.identity = known.or(cached.identity);
            return (held, true);
        }

        let text = match file.map(|_| std::fs::read_to_string(&self.path)) {
            None => None,
            Some(Ok(text)) => Some(text),
            Some(Err(_)) => return (self.held(&cached), false),
        };

        let read = Arc::new(match text {
            None => PublicationReading::nothing(&self.source()),
            Some(text) => {
                cached.revision += 1;
                PublicationReading::of(&text, cached.revision, &self.source())
            }
        });
        cached.stamp = file;
        cached.identity = known;
        cached.reading = Some(Arc::clone(&read));
        (read, true)
    }

    // What was read last, or nothing where nothing was.
    fn held(&self, cached: &Cached) -> Arc<PublicationReading> {
        cached
            .reading
            .clone()
            .unwrap_or_else(|| Arc::new(PublicationReading::nothing(&self.source())))
    }
}

// Which file is at `path`, asked only by a follow: the runtime's answer in
// a node's build; a remote one tells files apart by their stamp alone.
#[cfg(feature = "node")]
fn identity(path: &Path) -> Option<u128> {
    runtime::directory_watch::identity(path)
}

#[cfg(not(feature = "node"))]
fn identity(path: &Path) -> Option<u128> {
    stamp(path).map(|_| 0)
}

// The file's stamp, or None where no file is there: the runtime's, which
// leaves the file unopened, in a node's build; the standard library's in a
// remote one, the phone's, which reads a snapshot only where it is handed one.
#[cfg(feature = "node")]
fn stamp(path: &Path) -> Option<(u64, u64, u64)> {
    runtime::directory_watch::stamp(path).map(|at| (at.written, at.made, at.length))
}

#[cfg(not(feature = "node"))]
fn stamp(path: &Path) -> Option<(u64, u64, u64)> {
    let file = std::fs::metadata(path)
        .ok()
        .filter(std::fs::Metadata::is_file)?;
    let nanos = |at: std::io::Result<std::time::SystemTime>| {
        at.ok()
            .and_then(|at| at.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |since| {
                u64::try_from(since.as_nanos()).unwrap_or(u64::MAX)
            })
    };
    Some((nanos(file.modified()), nanos(file.created()), file.len()))
}

impl OperatorSurface for SnapshotSurface {
    fn source(&self) -> String {
        if self.exists() {
            format!("SNAPSHOT — {}", self.path.display())
        } else {
            format!("SNAPSHOT — no file at {}", self.path.display())
        }
    }

    fn health(&self, scope: &str) -> Vec<HealthRecord> {
        self.read().index.health(scope)
    }

    fn measure(&self, scope: &str, counted: Counted) -> Option<Count> {
        let now = i64::try_from(self.clock.unix_timestamp_nanos()).unwrap_or(i64::MAX);
        self.read().index.measure(scope, counted, now)
    }

    fn pause_scope(&self, _scope: &str, _who: &str) -> String {
        "a snapshot is a record of what was published and cannot be paused".to_string()
    }

    fn resume_scope(&self, _scope: &str) -> String {
        "a snapshot is a record of what was published and cannot be resumed".to_string()
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
        self.following.fetch_add(1, Ordering::AcqRel);
        crate::surface::snapshot_watch::watch(self, stop, each);
        self.following.fetch_sub(1, Ordering::AcqRel);
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

    fn act(&self, _node: &str, _noun: Noun, _target: &str, _act: Act, _who: &str) -> Acted {
        Acted::declined(NOT_ACTED)
    }

    fn act_message(&self, scope: &str, journey: &str, act: Act, who: &str) -> (String, Acted) {
        crate::surface::failed_message_list::acted_on_node(self, scope, journey, (act, who))
    }

    fn control(&self, scope: &str, action: ScopeAction, who: &str) -> ScopeOperation {
        let result = match action {
            ScopeAction::Pause => self.pause_scope(scope, who),
            ScopeAction::Resume => self.resume_scope(scope),
        };

        ScopeOperation {
            scope: scope.to_string(),
            action,
            applied: false,
            result,
        }
    }
}

impl SnapshotSurface {
    /// Read the file again where it changed: whether what is held is
    /// current.
    pub(crate) fn reread(&self) -> bool {
        self.refresh(true).1
    }
}

#[cfg(test)]
mod tests {
    use std::io::Read;
    use std::sync::Arc;

    use file::temporary::Folder;
    use observe::publication::write_atomic;
    use observe::{Health, HealthRecord, Publication, Snapshot};

    use super::SnapshotSurface;
    use crate::surface::OperatorSurface;
    use crate::surface::scope_tree::ROOT;

    // A publication of `records` records, as a publisher writes it.
    fn publication(records: usize) -> String {
        let cluster = configure::fixture::test_cluster();
        let mut snapshot = Snapshot::new();
        for record in 0..records {
            snapshot.record_health(HealthRecord {
                scope: format!("{}/send/p{record}", cluster.node_scope(0)),
                health: Health::Fine,
                severity: 0,
                evidence: String::new(),
                observed_unix_nanos: 1,
            });
        }
        Publication::whole("a test", &cluster.scope(), &snapshot).to_toml()
    }

    #[test]
    fn a_publication_renamed_over_a_file_being_read_is_read_whole_on_every_system() {
        // The publisher's collision with a reader: the next publication
        // renamed over the file a reader holds open (`write_atomic`). The
        // name is the next file's at once, whole, and the reader's open file
        // is the last, whole: neither is torn or an empty estate. A reader
        // whose open Windows refuses meanwhile keeps what it read last
        // (Xmip.Surface.Test, SnapshotOperatorTest).
        let folder = Folder::new("operate-snapshot-collision");
        let path = folder.join("snapshot.xmip");
        let last = publication(2);
        write_atomic(&path, &last).expect("published");
        let surface = SnapshotSurface::new(&path, Arc::new(xcore::SystemClock));
        assert_eq!(surface.health(ROOT).len(), 2);

        let mut reading = std::fs::File::open(&path).expect("opened");
        write_atomic(&path, &publication(3)).expect("renamed over the file being read");

        assert_eq!(surface.health(ROOT).len(), 3, "the next, whole");
        let mut read = String::new();
        reading.read_to_string(&mut read).expect("read");
        assert_eq!(read, last, "the last, whole, to the reader holding it");
    }
}
