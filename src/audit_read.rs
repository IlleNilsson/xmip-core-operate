//! The audit read back through the audit capability's one reader (ADR-0062,
//! amendment 2026-09-29), for every surface: the query in the words every
//! surface uses, the file a program told a directory reads — none where no
//! directory is stated, the records then in the operating system's log —
//! and what the capability found, with the columns and severities there are
//! to choose from. `xmip audit` reads it here; the C ABI hands it across as
//! `xmip_audit_read_v1`.

use std::path::Path;

use xaudit::audit_chain::Verdict;
use xaudit::audit_column::Column;
use xaudit::audit_entry::AuditEntry;
use xaudit::audit_fragment::Fragment;
use xaudit::audit_query::{AuditGroup, AuditQuery};
use xaudit::audit_store;
use xcore::Severity;

/// What a read found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AuditRead {
    /// The file read; empty where no directory is stated.
    pub file: String,
    /// How many records the file holds.
    pub read: usize,
    /// How many matched, of which `records` is the page asked for.
    pub matched: usize,
    pub offset: usize,
    pub limit: usize,
    pub records: Vec<AuditEntry>,
    pub groups: Vec<AuditGroup>,
    pub actions: Vec<String>,
    pub chains: Vec<Verdict>,
    pub fragments: Vec<Fragment>,
    /// The columns a read sorts by.
    pub columns: Vec<&'static str>,
    /// The severities a read filters by.
    pub severities: Vec<&'static str>,
}

/// Why a read found nothing to say.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unread {
    /// A query the capability does not take: its REFUSED sentence.
    Refused(String),
    /// The file is there and could not be read.
    Io(String),
}

/// The records `query` asks of the audit a program told `directory` keeps:
/// `directory` where one is stated, else the capability's rule
/// (`XMIP_AUDIT_DIRECTORY`).
///
/// # Errors
///
/// A query the capability refuses, or a file it could not read.
pub fn read(directory: Option<&Path>, query: &[(String, String)]) -> Result<AuditRead, Unread> {
    let asked = AuditQuery::from_pairs(
        query
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    )
    .map_err(Unread::Refused)?;
    let mut read = AuditRead {
        offset: asked.offset,
        limit: asked.limit,
        columns: Column::ALL.map(Column::word).to_vec(),
        severities: Severity::ALL.map(Severity::word).to_vec(),
        ..AuditRead::default()
    };

    let Some(file) = audit_store::stated(directory) else {
        return Ok(read);
    };
    // A page is read from the file a page at a time; a verification walks
    // the file's bytes as they are now (ADR-0070 clause 5), and says the
    // bytes no write finished apart from the chains.
    let page = audit_store::ask(&file, &asked).map_err(|error| Unread::Io(error.to_string()))?;

    read.file = file.display().to_string();
    read.read = page.read;
    read.matched = page.matched;
    read.records = page.records;
    read.groups = page.groups;
    read.actions = page.actions;
    read.chains = page.chains;
    read.fragments = page.fragments;

    Ok(read)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_not_made_yet_holds_nothing_and_a_query_it_does_not_take_is_refused() {
        // A directory that is no folder yet: nothing has been audited there.
        let nowhere = std::env::temp_dir().join(format!("xmip-audit-read-{}", std::process::id()));
        let read = read(Some(&nowhere), &[]).expect("read");

        assert!(read.file.ends_with("audit.toml"));
        assert_eq!((read.read, read.matched), (0, 0));
        assert!(!read.columns.is_empty() && !read.severities.is_empty());

        let refused = super::read(Some(&nowhere), &[("color".to_string(), "red".to_string())]);
        assert!(matches!(refused, Err(Unread::Refused(said)) if said.starts_with("REFUSED")));
    }
}
