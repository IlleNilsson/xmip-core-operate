//! A host's `[Xmip]` table, in the words it is read and written by: the
//! choice reads a document's surface by them ([`super::decided`]), and
//! [`local_document`] writes the one an install leaves beside the command
//! line, [`COMMAND_LINE`], stating the node on this machine it installed
//! (`xmip-service --install`). Both go through configure's one reader and
//! writer (`configure::document`), so what is written is what is read.

use toml::{Table, Value};

/// The host's table.
pub(super) const TABLE: &str = "Xmip";

/// The key naming the surface a table reads.
pub(super) const SURFACE: &str = "Surface";

/// The surface a node on this machine is, as [`SURFACE`] says it.
pub(super) const LOCAL: &str = "local";

/// The key naming that node's local endpoint.
pub(super) const ENDPOINT: &str = "Local";

/// The command line's own document, beside `xmip` (ADR-0052 clause 3).
pub const COMMAND_LINE: &str = "cli.toml";

/// A host's document stating the node on this machine at its local
/// endpoint `endpoint`, and nothing else: no audit directory, so the audit
/// capability decides.
///
/// # Errors
///
/// The document does not write, in configure's words.
pub fn local_document(endpoint: &str) -> Result<String, String> {
    let mut table = Table::new();
    table.insert(SURFACE.to_string(), Value::from(LOCAL));
    table.insert(ENDPOINT.to_string(), Value::from(endpoint));
    let mut document = Table::new();
    document.insert(TABLE.to_string(), Value::Table(table));

    configure::document::write(&document)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::choice::{Asked, Choice, Document, SurfaceLine, decided};

    #[test]
    fn the_local_document_reads_back_as_the_node_on_this_machine_at_its_endpoint() {
        // Whatever `[operate] local` names: the test node's own name.
        let endpoint = configure::fixture::test_cluster().node(0).name.clone();
        let written = local_document(&endpoint).expect("writes");
        let table = configure::document::read(&written).expect("TOML")[TABLE]
            .as_table()
            .cloned()
            .expect("its table");
        let base = file::temporary::Folder::new("operate-table");
        let document = Document::of(table, base.to_path_buf());

        assert_eq!(
            decided(&Asked::Stated(SurfaceLine::default()), &document, false),
            Ok(Choice::Local(Some(endpoint.clone())))
        );
        assert_eq!(
            decided(&Asked::Host, &document, false),
            Ok(Choice::Local(Some(endpoint)))
        );
        assert_eq!(document.value("AuditDirectory"), None);
    }
}
