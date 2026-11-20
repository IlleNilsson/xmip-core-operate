//! The surface a command, a cmdlet or the prompt reads, chosen and never
//! guessed (ADR-0052 clause 3): what the line states for one invocation —
//! `--remote`, then `--local`, then `--snapshot` — over the host's
//! document, whose `[Xmip]` table says `Surface = "local" | "snapshot" |
//! "remote"` with its `Local`, `Snapshot` or `Address`, or names the first
//! of its `Clusters`. A remote presents the certificate the document's
//! `Certificate` and `PrivateKey` name and trusts the anchors `TrustAnchor`
//! names, each else its `XMIP_` variable (ADR-0063; [`named`]). What is
//! chosen is decided once here ([`decided`]); a surface in Rust opens it
//! ([`open`]), one in another language opens what the decision names. The
//! table's words, and the document an install writes in them, are [`table`].

pub mod named;
pub mod table;

use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use xcore::Clock;

use crate::surface::OperatorSurface;
use crate::surface::listener::{ListenerSurface, remote_refusal};
use crate::surface::snapshot::SnapshotSurface;
use named::Named;
use table::{ENDPOINT, LOCAL, SURFACE, TABLE};

/// What an operator stated for one invocation about the surface to read.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SurfaceLine {
    pub remote: Option<String>,
    pub local: Option<String>,
    pub snapshot: Option<String>,
}

/// A host's document, read: its `[Xmip]` table, keys found whatever their
/// case, and the folder its paths are relative to.
#[derive(Clone, Debug, Default)]
pub struct Document {
    table: toml::Table,
    /// The folder the document is in.
    pub base: PathBuf,
    /// The environment a host handed, read in place of this process's:
    /// a .NET host's variables never reach the process's own on Unix.
    environment: Option<std::collections::BTreeMap<String, String>>,
}

impl Document {
    /// The document at `path`, read by configure's one reader
    /// (`configure::document`); empty where there is none or it is no TOML.
    #[must_use]
    pub fn read(path: &Path) -> Self {
        let base = resolve(path, Path::new("."))
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let whole = std::fs::read_to_string(path)
            .ok()
            .and_then(|text| configure::document::read(&text).ok())
            .unwrap_or_default();
        let table = found(&whole, TABLE)
            .and_then(toml::Value::as_table)
            .cloned()
            .unwrap_or_default();

        Self {
            table,
            base,
            environment: None,
        }
    }

    /// A host's `[Xmip]` table its own reader read, its paths beneath
    /// `base`.
    #[must_use]
    pub fn of(table: toml::Table, base: PathBuf) -> Self {
        Self {
            table,
            base,
            environment: None,
        }
    }

    /// This document with the variables a host handed read in place of the
    /// process's environment.
    #[must_use]
    pub fn with_environment(mut self, handed: std::collections::BTreeMap<String, String>) -> Self {
        self.environment = Some(handed);
        self
    }

    /// The `[Xmip]` key `key`, as text; `None` where it is not text.
    #[must_use]
    pub fn value(&self, key: &str) -> Option<&str> {
        found(&self.table, key).and_then(toml::Value::as_str)
    }

    /// `key`, else the environment variable `variable`, resolved against
    /// the document's folder; `None` where neither names one.
    #[must_use]
    pub fn named(&self, key: &str, variable: &str) -> Option<PathBuf> {
        let named = self
            .value(key)
            .filter(|named| !named.trim().is_empty())
            .map(str::to_string)
            .or_else(|| match &self.environment {
                Some(handed) => handed.get(variable).cloned(),
                None => std::env::var(variable).ok(),
            })
            .filter(|named| !named.trim().is_empty())?;

        Some(resolve(Path::new(&named), &self.base))
    }

    /// The files a remote surface presents and trusts, as the document and
    /// the environment name them, checked; `keeps` where a keeper holds the
    /// key.
    ///
    /// # Errors
    ///
    /// As [`Named::checked`].
    pub fn identity_named(&self, keeps: bool) -> Result<Named, String> {
        Named::checked(
            self.named("Certificate", "XMIP_CERTIFICATE"),
            self.named("PrivateKey", "XMIP_PRIVATE_KEY"),
            self.named("TrustAnchor", "XMIP_TRUST_ANCHOR"),
            keeps,
        )
    }

    fn clusters(&self) -> Vec<toml::Table> {
        found(&self.table, "Clusters")
            .and_then(toml::Value::as_array)
            .map(|clusters| {
                clusters
                    .iter()
                    .filter_map(|one| one.as_table().cloned())
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Which surface is asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Asked {
    /// What a line states over the document: the line's surface, else the
    /// first of `Clusters`, else the table's, else a local surface naming
    /// no node.
    Stated(SurfaceLine),
    /// The surface the host's `[Xmip]` table names itself.
    Host,
    /// The surface the cluster at this place of `Clusters` names.
    Cluster(usize),
}

/// A surface decided, before it is opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Choice {
    /// A published snapshot at this path.
    Snapshot(PathBuf),
    /// A node on this machine at this local endpoint; none names no node.
    Local(Option<String>),
    /// A node at this operate address, presenting and trusting what is
    /// named.
    Remote { address: String, named: Named },
}

/// The surface `asked` names over `document`, `keeps` where a keeper holds
/// this end's key.
///
/// # Errors
///
/// A document that names a surface it does not say enough of, or files it
/// does not have: why, in the table's own keys.
pub fn decided(asked: &Asked, document: &Document, keeps: bool) -> Result<Choice, String> {
    match asked {
        Asked::Stated(line) => stated_choice(line, document, keeps),
        Asked::Host => table_choice(&document.table, TABLE, document, keeps),
        Asked::Cluster(at) => document.clusters().get(*at).map_or_else(
            || Err(format!("{TABLE}:Clusters names no cluster at {at}")),
            |cluster| table_choice(cluster, &format!("{TABLE}:Clusters:{at}"), document, keeps),
        ),
    }
}

fn stated_choice(line: &SurfaceLine, document: &Document, keeps: bool) -> Result<Choice, String> {
    let given = |value: &Option<String>| value.clone().filter(|value| !value.trim().is_empty());

    if let Some(address) = given(&line.remote) {
        let named = document.identity_named(keeps)?;
        return Ok(Choice::Remote { address, named });
    }
    if let Some(local) = given(&line.local) {
        return Ok(Choice::Local(Some(local)));
    }
    if let Some(snapshot) = given(&line.snapshot) {
        return Ok(Choice::Snapshot(resolve(
            Path::new(&snapshot),
            &document.base,
        )));
    }

    let surface = document
        .value(SURFACE)
        .filter(|chosen| !chosen.trim().is_empty());

    match (document.clusters().first(), surface) {
        (Some(first), _) => table_choice(first, &format!("{TABLE}:Clusters:0"), document, keeps),
        (None, Some(_)) => table_choice(&document.table, TABLE, document, keeps),
        (None, None) => Ok(Choice::Local(None)),
    }
}

// The one surface a table names, refused in the table's own keys.
fn table_choice(
    table: &toml::Table,
    at: &str,
    document: &Document,
    keeps: bool,
) -> Result<Choice, String> {
    let text = |key: &str| {
        found(table, key)
            .and_then(toml::Value::as_str)
            .map(str::to_string)
    };
    let chosen = text(SURFACE)
        .map(|chosen| chosen.trim().to_string())
        .unwrap_or_default();
    let key = format!("{at}:{SURFACE}");

    if chosen.eq_ignore_ascii_case(LOCAL) {
        return Ok(Choice::Local(text(ENDPOINT)));
    }
    if chosen.eq_ignore_ascii_case("snapshot") {
        return match text("Snapshot").filter(|path| !path.trim().is_empty()) {
            Some(path) => Ok(Choice::Snapshot(resolve(Path::new(&path), &document.base))),
            None => Err(format!(
                "{key} is \"snapshot\" and {at}:Snapshot names no file"
            )),
        };
    }
    if chosen.eq_ignore_ascii_case("remote") {
        let address = text("Address");
        return match remote_refusal(address.as_deref()) {
            None => Ok(Choice::Remote {
                address: address.unwrap_or_default(),
                named: document.identity_named(keeps)?,
            }),
            Some(_) => Err(format!(
                "{key} is \"remote\" and {at}:Address names no node"
            )),
        };
    }

    let words = "\"local\", \"snapshot\" or \"remote\"";
    Err(if chosen.is_empty() {
        format!("{key} is not set; it is {words}")
    } else {
        format!("{key} is \"{chosen}\"; it is {words}")
    })
}

/// The surface `choice` names, opened, telling time by `clock`; a kept key
/// is opened by whoever holds the keeper (`ffi::surface`).
#[must_use]
pub fn open(choice: Choice, clock: &Arc<dyn Clock>) -> Chosen {
    match choice {
        Choice::Snapshot(path) => Chosen::Snapshot(SnapshotSurface::new(path, Arc::clone(clock))),
        Choice::Local(endpoint) => Chosen::Listener(ListenerSurface::local(
            endpoint.as_deref(),
            Arc::clone(clock),
        )),
        Choice::Remote { address, named } => Chosen::Listener(ListenerSurface::remote(
            &address,
            named.identity(),
            Arc::clone(clock),
        )),
    }
}

/// The surface `line` states over `document`.
///
/// # Errors
///
/// As [`decided`].
pub fn stated(
    line: &SurfaceLine,
    document: &Document,
    clock: &Arc<dyn Clock>,
) -> Result<Chosen, String> {
    decided(&Asked::Stated(line.clone()), document, false).map(|choice| open(choice, clock))
}

/// The surface `line` states over the document at `path`, where it
/// answers.
///
/// # Errors
///
/// Why it cannot be read: the document's refusal, named by its file, or
/// the surface's own source saying why it does not answer.
pub fn answering(
    line: &SurfaceLine,
    path: &Path,
    clock: &Arc<dyn Clock>,
) -> Result<Chosen, String> {
    let document = Document::read(path);
    let named = path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into());
    let chosen = stated(line, &document, clock).map_err(|refused| format!("{named}: {refused}"))?;

    if chosen.answers() {
        Ok(chosen)
    } else {
        Err(chosen.source())
    }
}

/// The surface a line and a document chose.
pub enum Chosen {
    Snapshot(SnapshotSurface),
    Listener(ListenerSurface),
}

impl Chosen {
    /// The surface itself.
    #[must_use]
    pub fn surface(&self) -> &(dyn OperatorSurface + Send + Sync) {
        match self {
            Self::Snapshot(snapshot) => snapshot,
            Self::Listener(node) => node,
        }
    }

    /// Its source.
    #[must_use]
    pub fn source(&self) -> String {
        self.surface().source()
    }

    /// What it answers from now (`PublicationReading`).
    #[must_use]
    pub fn reading(&self) -> Arc<crate::surface::publication_reading::PublicationReading> {
        match self {
            Self::Snapshot(snapshot) => snapshot.reading(),
            Self::Listener(node) => node.reading(),
        }
    }

    /// Whether it answers: a node that answers now, a snapshot whose file
    /// is there.
    #[must_use]
    pub fn answers(&self) -> bool {
        match self {
            Self::Listener(node) => node.connect(),
            Self::Snapshot(snapshot) => snapshot.exists(),
        }
    }
}

// A key of `table`, whatever its case, as a host's document reads keys.
fn found<'a>(table: &'a toml::Table, key: &str) -> Option<&'a toml::Value> {
    table.get(key).or_else(|| {
        table
            .iter()
            .find(|(named, _)| named.eq_ignore_ascii_case(key))
            .map(|(_, value)| value)
    })
}

/// `path`, absolute: as it is where it is rooted, else beneath `base`, its
/// `.` and `..` taken away.
#[must_use]
pub fn resolve(path: &Path, base: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::path::absolute(base)
            .unwrap_or_else(|_| base.to_path_buf())
            .join(path)
    };
    let mut resolved = PathBuf::new();

    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            other => resolved.push(other),
        }
    }

    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_is_resolved_beneath_its_base_with_no_dots_left() {
        let folder = file::temporary::Folder::new("operate-resolve");
        let base = folder.join("a").join("b");
        let resolved = resolve(Path::new("../c/./d.xmip"), &base);

        assert_eq!(resolved, folder.join("a").join("c").join("d.xmip"));
    }

    #[test]
    fn a_table_names_its_surface_or_is_refused_in_its_own_keys() {
        let base = file::temporary::Folder::new("operate-surface-named");
        let document = Document::of(
            "Surface = \"elsewhere\"".parse().expect("TOML"),
            base.to_path_buf(),
        );
        let clock: Arc<dyn Clock> = Arc::new(xcore::SystemClock);

        let refused = stated(&SurfaceLine::default(), &document, &clock)
            .err()
            .expect("refused");

        assert_eq!(
            refused,
            "Xmip:Surface is \"elsewhere\"; it is \"local\", \"snapshot\" or \"remote\""
        );
        assert_eq!(document.value("surface"), Some("elsewhere"));
    }

    #[test]
    fn a_line_wins_over_the_document_and_the_first_cluster_over_its_table() {
        let table: toml::Table = "Surface = \"local\"\nLocal = \"xmip-a\"\n\
            [[Clusters]]\nSurface = \"snapshot\"\nSnapshot = \"c.xmip\""
            .parse()
            .expect("TOML");
        let folder = file::temporary::Folder::new("operate-line-wins");
        let base = folder.to_path_buf();
        let document = Document::of(table, base.clone());
        let line = SurfaceLine {
            local: Some("xmip-b".into()),
            ..SurfaceLine::default()
        };

        assert_eq!(
            decided(&Asked::Stated(line), &document, false),
            Ok(Choice::Local(Some("xmip-b".into())))
        );
        assert_eq!(
            decided(&Asked::Stated(SurfaceLine::default()), &document, false),
            Ok(Choice::Snapshot(resolve(Path::new("c.xmip"), &base)))
        );
        assert_eq!(
            decided(&Asked::Host, &document, false),
            Ok(Choice::Local(Some("xmip-a".into())))
        );
    }
}
