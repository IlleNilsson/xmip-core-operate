use std::path::{Path, PathBuf};

use abi::ffi::status;
use abi::operate::publication::Publication as PublicationHandle;
use abi::operate::scope_index::ScopeIndex as IndexHandle;
use abi::operate::surface as declared;
use observe::{Health, HealthRecord, Publication, Snapshot};
use serde_json::Value;

use super::asked::xmip_surface_set_secret_v1;
use super::asked::{xmip_surface_act_message_v1, xmip_surface_secrets_v1};
use super::asked::{xmip_surface_act_v1, xmip_surface_control_v1, xmip_surface_failed_v1};
use super::chosen::xmip_surface_unnamed_v1;
use super::chosen::{xmip_surface_decided_v1, xmip_surface_named_v1, xmip_surface_unlisted_v1};
use super::chosen::{xmip_surface_local_refusal_v1, xmip_surface_not_acted_v1};
use super::chosen::{xmip_surface_port_at_v1, xmip_surface_remote_refusal_v1};
use super::opened::{xmip_surface_answers_v1, xmip_surface_local_v1, xmip_surface_snapshot_v1};
use super::opened::{xmip_surface_remote_kept_v1, xmip_surface_remote_v1, xmip_surface_source_v1};
use super::read::xmip_surface_read_v1;
use super::{xmip_surface_free_v1, xmip_surface_wait_v1};
use crate::ffi::publication::xmip_publication_free_v1;
use crate::ffi::scope_index::xmip_scope_index_free_v1;
use crate::ffi::text::borrow;
use file::temporary::Folder;

#[test]
fn every_export_has_the_shape_the_binding_declares() {
    let _: declared::OpenFn = xmip_surface_snapshot_v1;
    let _: declared::OpenFn = xmip_surface_local_v1;
    let _: declared::RemoteFn = xmip_surface_remote_v1;
    let _: declared::RemoteKeptFn = xmip_surface_remote_kept_v1;
    let _: declared::FreeFn = xmip_surface_free_v1;
    let _: declared::AnswersFn = xmip_surface_answers_v1;
    let _: declared::ReadFn = xmip_surface_read_v1;
    let _: declared::WaitFn = xmip_surface_wait_v1;
    for asking in [
        xmip_surface_source_v1 as declared::AskFn,
        xmip_surface_act_v1,
        xmip_surface_act_message_v1,
        xmip_surface_control_v1,
        xmip_surface_failed_v1,
        xmip_surface_secrets_v1,
        xmip_surface_set_secret_v1,
        super::asked::xmip_surface_standing_v1,
    ] {
        let _ = asking;
    }
    for said in [
        xmip_surface_decided_v1 as declared::SaidFn,
        xmip_surface_named_v1,
        xmip_surface_unlisted_v1,
        xmip_surface_port_at_v1,
        xmip_surface_remote_refusal_v1,
        xmip_surface_local_refusal_v1,
        xmip_surface_not_acted_v1,
        xmip_surface_unnamed_v1,
    ] {
        let _ = said;
    }
}

// The test cluster's scope and its first node's.
fn cluster() -> String {
    configure::fixture::test_cluster().scope()
}

fn node() -> String {
    configure::fixture::test_cluster().node_scope(0)
}

/// A publication saying `evidence`, written as `<name>.xmip` in `folder`.
fn published(folder: &Path, name: &str, evidence: &str) -> PathBuf {
    let mut snapshot = Snapshot::new();
    snapshot.record_health(HealthRecord {
        scope: format!("{}/send/p", node()),
        health: Health::Exhausted,
        severity: 40,
        evidence: evidence.into(),
        observed_unix_nanos: 1,
    });
    let path = folder.join(format!("{name}.xmip"));
    let text = Publication::whole("a test", &cluster(), &snapshot).to_toml();
    std::fs::write(&path, text).expect("published");
    path
}

fn said(export: declared::SaidFn, input: &str, argument: &str) -> (i32, String) {
    let mut out = vec![0_u8; 4096];
    let mut len = 0_usize;
    // SAFETY: two borrowed strings, a buffer of its stated room and a
    // writable length.
    let code = unsafe {
        export(
            borrow(input),
            borrow(argument),
            out.as_mut_ptr(),
            out.len(),
            &raw mut len,
        )
    };
    out.truncate(len);
    (code, String::from_utf8(out).expect("UTF-8"))
}

fn asked(
    export: declared::AskFn,
    surface: *const declared::Surface,
    input: &str,
    argument: &str,
) -> (i32, String) {
    let mut out = vec![0_u8; 4096];
    let mut len = 0_usize;
    // SAFETY: a live surface, two borrowed strings, a buffer of its stated
    // room and a writable length.
    let code = unsafe {
        export(
            surface,
            0,
            borrow(input),
            borrow(argument),
            out.as_mut_ptr(),
            out.len(),
            &raw mut len,
        )
    };
    out.truncate(len);
    (code, String::from_utf8(out).expect("UTF-8"))
}

fn snapshot_over(path: &std::path::Path) -> *mut declared::Surface {
    let text = path.display().to_string();
    let mut surface = core::ptr::null_mut();
    // SAFETY: a borrowed path and a writable handle.
    assert_eq!(
        unsafe { xmip_surface_snapshot_v1(borrow(&text), 0, &raw mut surface) },
        status::OK
    );
    surface
}

#[test]
fn a_snapshot_is_read_once_handed_with_its_tree_and_never_acted_on() {
    let folder = Folder::new("operate-surface-read");
    let path = published(&folder, "read", "full");
    let surface = snapshot_over(&path);
    let mut answers = 0_u8;
    // SAFETY: a live surface and a writable answer.
    assert_eq!(
        unsafe { xmip_surface_answers_v1(surface, &raw mut answers) },
        status::OK
    );
    assert_eq!(answers, 1);

    let read = |seen: u64| {
        let (mut publication, mut index): (*mut PublicationHandle, *mut IndexHandle) =
            (core::ptr::null_mut(), core::ptr::null_mut());
        let mut out = vec![0_u8; 4096];
        let mut len = 0_usize;
        // SAFETY: a live surface, writable handles and a buffer.
        let code = unsafe {
            xmip_surface_read_v1(
                surface,
                seen,
                &raw mut publication,
                &raw mut index,
                out.as_mut_ptr(),
                out.len(),
                &raw mut len,
            )
        };
        out.truncate(len);
        let head: Value = if len == 0 {
            Value::Null
        } else {
            serde_json::from_slice(&out).expect("JSON")
        };
        (code, head, publication, index)
    };

    let (code, head, publication, index) = read(0);
    assert_eq!(code, status::OK);
    assert_eq!(head["changed"], true);
    assert_eq!(head["root"], cluster());
    assert!(!publication.is_null() && !index.is_null());
    // SAFETY: handles this section handed out, freed once.
    unsafe {
        xmip_publication_free_v1(publication);
        xmip_scope_index_free_v1(index);
    }

    let reading = head["reading"].as_u64().expect("a reading");
    let (_, again, publication, index) = read(reading);
    assert_eq!(again, Value::Null, "an unchanged reading says nothing");
    assert!(publication.is_null() && index.is_null());

    let act = serde_json::json!({
        "node": node(), "noun": "subscription", "target": "s", "act": "pause",
    })
    .to_string();
    let (code, acted) = asked(xmip_surface_act_v1, surface, &act, "");
    let acted: Value = serde_json::from_str(&acted).expect("JSON");
    assert_eq!(
        (code, acted["outcome"].as_i64()),
        (
            status::OK,
            Some(i64::from(abi::operate::words::outcome::REFUSED))
        )
    );
    assert_eq!(acted["said"], crate::surface::snapshot::NOT_ACTED);

    let pause = serde_json::json!({ "scope": cluster(), "action": "pause", "who": "w" });
    let (_, control) = asked(xmip_surface_control_v1, surface, &pause.to_string(), "");
    assert!(control.contains(r#""applied":false"#), "{control}");

    // SAFETY: a handle this section handed out, freed once.
    unsafe { xmip_surface_free_v1(surface) };
}

#[test]
fn a_follow_tells_at_once_then_each_replacement() {
    let folder = Folder::new("operate-surface-follow");
    let path = published(&folder, "follow", "first");
    let surface = snapshot_over(&path);
    let (mut told, mut revision) = (0_u64, 0_u64);
    // SAFETY: a live surface and writable answers.
    assert_eq!(
        unsafe { xmip_surface_wait_v1(surface, 0, 5_000, &raw mut told, &raw mut revision) },
        status::OK
    );
    assert_eq!(told, 1, "told once at once");

    let next = folder.join("next.xmip");
    std::fs::copy(published(&folder, "follow-next", "second"), &next).expect("copied");
    std::fs::rename(&next, &path).expect("replaced");
    // SAFETY: as above.
    assert_eq!(
        unsafe { xmip_surface_wait_v1(surface, 1, 5_000, &raw mut told, &raw mut revision) },
        status::OK
    );
    assert!(told >= 2, "the replacement is told: {told}");

    // SAFETY: a handle this section handed out, freed once.
    unsafe { xmip_surface_free_v1(surface) };
}

#[test]
fn a_follow_begun_before_its_directory_is_made_tells_once_it_is() {
    // The publications copied in are written apart, outside the folder
    // followed.
    let sources = Folder::new("operate-surface-sources");
    let place = Folder::new("operate-surface-arrival");
    let directory = place.join("not").join("made");
    let path = directory.join("snapshot.xmip");
    let surface = snapshot_over(&path);
    let (mut told, mut revision) = (0_u64, 0_u64);
    // SAFETY: a live surface and writable answers.
    assert_eq!(
        unsafe { xmip_surface_wait_v1(surface, 0, 5_000, &raw mut told, &raw mut revision) },
        status::OK
    );
    assert_eq!(told, 1, "told at once that nothing is there");

    std::fs::create_dir_all(&directory).expect("made");
    std::fs::copy(published(&sources, "arrival", "first"), &path).expect("landed");
    let mut seen = told;
    // SAFETY: as above.
    assert_eq!(
        unsafe { xmip_surface_wait_v1(surface, seen, 5_000, &raw mut told, &raw mut revision) },
        status::OK
    );
    assert!(told > seen, "the arrival is told: {told}");

    // A replacement after it is told as any follow tells one; the arrival
    // may have been told twice, the folder's and the file's.
    // SAFETY: as above.
    unsafe { xmip_surface_wait_v1(surface, told, 300, &raw mut told, &raw mut revision) };
    seen = told;
    let next = directory.join("snapshot.xmip.next");
    let second = published(&sources, "arrival-next", "second");
    std::fs::copy(&second, &next).expect("copied");
    std::fs::rename(&next, &path).expect("replaced");
    // SAFETY: as above.
    assert_eq!(
        unsafe { xmip_surface_wait_v1(surface, seen, 5_000, &raw mut told, &raw mut revision) },
        status::OK
    );
    assert!(told > seen, "the replacement is told: {told}");

    // A publication again, byte for byte the last: the copy keeps its
    // write time and length, and Windows gives the name's creation time
    // back to it, yet it is another file, and a node's build tells it. A
    // remote build watches no directory and tells files apart by their
    // stamp alone.
    #[cfg(feature = "node")]
    {
        seen = told;
        std::fs::copy(&second, &next).expect("copied");
        std::fs::rename(&next, &path).expect("replaced");
        // SAFETY: as above.
        assert_eq!(
            unsafe { xmip_surface_wait_v1(surface, seen, 5_000, &raw mut told, &raw mut revision) },
            status::OK
        );
        assert!(told > seen, "the same publication again is told: {told}");
    }

    // SAFETY: a handle this section handed out, freed once.
    unsafe { xmip_surface_free_v1(surface) };
}

#[test]
fn a_local_surface_naming_no_node_says_so_and_answers_nothing() {
    let mut surface = core::ptr::null_mut();
    // SAFETY: an empty endpoint and a writable handle.
    assert_eq!(
        unsafe { xmip_surface_local_v1(borrow(""), 0, &raw mut surface) },
        status::OK
    );
    let (_, source) = asked(xmip_surface_source_v1, surface, "", "");
    assert!(
        source.ends_with(crate::surface::listener::UNNAMED),
        "{source}"
    );
    let (code, refused) = asked(xmip_surface_secrets_v1, surface, &node(), "");
    assert_eq!((code, refused), (status::INVALID, source));
    // SAFETY: a handle this section handed out, freed once.
    unsafe { xmip_surface_free_v1(surface) };
}

#[test]
fn a_line_over_a_document_is_decided_in_the_table_s_own_words() {
    let line = r#"{"asked":"line","base":"/b"}"#;
    let (code, chosen) = said(
        xmip_surface_decided_v1,
        line,
        r#"{"Surface":"local","Local":"xmip-a"}"#,
    );
    // Read as JSON, never compared as text: a map's order is serde_json's,
    // and another crate in the estate's one workspace may turn on its
    // `preserve_order`.
    assert_eq!(code, status::OK);
    assert_eq!(
        serde_json::from_str::<Value>(&chosen).expect("JSON"),
        serde_json::json!({ "endpoint": "xmip-a", "surface": "local" })
    );

    let (code, refused) = said(
        xmip_surface_decided_v1,
        r#"{"asked":"host"}"#,
        r#"{"Surface":"x"}"#,
    );
    assert_eq!(code, status::INVALID);
    assert_eq!(
        refused,
        "Xmip:Surface is \"x\"; it is \"local\", \"snapshot\" or \"remote\""
    );

    let (code, named) = said(xmip_surface_named_v1, r#"{"certificate":"c.pem"}"#, "");
    assert_eq!(code, status::INVALID);
    assert!(named.contains("named together"), "{named}");

    let (_, unlisted) = said(
        xmip_surface_unlisted_v1,
        r#"{"listed":false,"scope":"s","source":"src"}"#,
        "",
    );
    assert!(unlisted.starts_with("NOT LISTED: src"), "{unlisted}");
    let at = node();
    let (_, port) = said(xmip_surface_port_at_v1, &format!("{at}/send/p/x"), "");
    assert_eq!(
        serde_json::from_str::<Value>(&port).expect("JSON"),
        serde_json::json!({ "node": at.as_str(), "port": "p" })
    );
    assert_eq!(said(xmip_surface_remote_refusal_v1, "node1:7401", "").1, "");
    assert!(
        said(xmip_surface_local_refusal_v1, " ", "")
            .1
            .starts_with("A local node")
    );
}
