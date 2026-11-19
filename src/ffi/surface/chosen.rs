//! Section 28.2's pure exports, in section 10's shape: which surface a line
//! and a host's document choose, whether the files a remote presents are
//! named as they must be, and the sentences a surface says — a list of
//! failed Messages that is none, a line's option with nothing after it, a
//! snapshot asked to act, a local surface naming no node — and the node and
//! Send Port a scope is at.

use std::path::{Path, PathBuf};

use abi::ffi::Str;
use serde_json::{Value, json};

use crate::ffi::text::answer;
use crate::surface::choice::named::Named;
use crate::surface::choice::{self, Asked, Choice, Document, SurfaceLine};
use crate::surface::failed_message_list::{FailedMessageList, port_at};
use crate::surface::listener::{UNNAMED, local_refusal, remote_refusal};
use crate::surface::snapshot::NOT_ACTED;

/// The surface asked for, `input` saying how — `{"asked":"line"|"host"|
/// "cluster","cluster":n,"remote","local","snapshot","kept","base"}` — over
/// the host's `[Xmip]` table, `argument`, as its own reader read it: the
/// choice as JSON, `{"surface":"snapshot","path"}`,
/// `{"surface":"local","endpoint"}` (null: none named) or
/// `{"surface":"remote","address","certificate","key","anchors","kept"}`;
/// the refusal, in the table's own keys, where the document does not say
/// enough.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_decided_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, table| {
            let asked: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
            let text = |name: &str| asked[name].as_str().map(str::to_string);
            let base = PathBuf::from(text("base").unwrap_or_default());
            let table = if table.trim().is_empty() {
                toml::Table::new()
            } else {
                crate::document::table_of(table)?
            };
            let document = handed(Document::of(table, base), &asked["environment"]);
            let how = match text("asked").as_deref() {
                Some("host") => Asked::Host,
                Some("cluster") => Asked::Cluster(
                    usize::try_from(asked["cluster"].as_u64().unwrap_or(0)).unwrap_or(usize::MAX),
                ),
                _ => Asked::Stated(SurfaceLine {
                    remote: text("remote"),
                    local: text("local"),
                    snapshot: text("snapshot"),
                }),
            };
            let keeps = asked["kept"].as_bool().unwrap_or(false);
            choice::decided(&how, &document, keeps).map(|choice| decided(&choice).to_string())
        })
    }
}

/// The files a remote surface presents and trusts, checked: the input
/// `{"certificate","key","anchors","kept"}` — paths, null where none is
/// named; `kept` where a keeper holds the key — or, where the argument is a
/// host's `[Xmip]` table as JSON, the input `{"base","environment","kept"}`
/// and the files the table names, else the environment's variables,
/// beneath `base`. The answer `{"certificate","key","anchors","kept"}`, or
/// the refusal where they are not named together or one is not there.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_named_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, table| {
            let asked: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
            let keeps = asked["kept"].as_bool().unwrap_or(false);
            let named = if table.trim().is_empty() {
                let path = |name: &str| asked[name].as_str().map(PathBuf::from);
                Named::checked(path("certificate"), path("key"), path("anchors"), keeps)?
            } else {
                let base = PathBuf::from(asked["base"].as_str().unwrap_or_default());
                let document = Document::of(crate::document::table_of(table)?, base);
                handed(document, &asked["environment"]).identity_named(keeps)?
            };
            let shown = |path: &Option<PathBuf>| path.as_ref().map(|at| at.display().to_string());
            Ok(json!({
                "certificate": shown(&named.certificate),
                "key": shown(&named.key),
                "anchors": shown(&named.anchors),
                "kept": named.kept,
            })
            .to_string())
        })
    }
}

// `document` with the variables a host handed, an object of names and
// texts, read in place of this process's environment; as it is where none
// was handed.
fn handed(document: Document, environment: &Value) -> Document {
    match environment.as_object() {
        Some(variables) => document.with_environment(
            variables
                .iter()
                .filter_map(|(name, value)| Some((name.clone(), value.as_str()?.to_string())))
                .collect(),
        ),
        None => document,
    }
}

/// What a list of failed Messages that is none says, the input
/// `{"listed","failure","scope","source"}`: `NOT LISTED:` and why, or its
/// `FAILED:`; empty for a list.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_unlisted_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, _| {
            let asked: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
            let text = |name: &str| asked[name].as_str().unwrap_or_default().to_string();
            let list = FailedMessageList {
                ports: Vec::new(),
                listed: asked["listed"].as_bool().unwrap_or(false),
                failure: text("failure"),
            };
            Ok(list.unlisted_said(&text("scope"), &text("source")))
        })
    }
}

/// The node's own scope the input is on, and the Send Port where it is on
/// a stage beneath it: `{"node","port"}`, each empty where there is none.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_port_at_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |scope, _| {
            let (node, port) = port_at(scope);
            Ok(json!({ "node": node, "port": port }).to_string())
        })
    }
}

/// Why a line naming `--remote` with the input after it cannot be obeyed;
/// empty where it can.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_remote_refusal_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |address, _| {
            Ok(remote_refusal(Some(address))
                .unwrap_or_default()
                .to_string())
        })
    }
}

/// Why a line naming `--local` with the input after it cannot be obeyed;
/// empty where it can.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_local_refusal_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |endpoint, _| {
            Ok(local_refusal(Some(endpoint))
                .unwrap_or_default()
                .to_string())
        })
    }
}

/// Why a snapshot takes no act.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_not_acted_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |_, _| {
            Ok(NOT_ACTED.to_string())
        })
    }
}

/// What a local surface naming no node says on every answer.
///
/// # Safety
/// As section 10's exports.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_unnamed_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |_, _| {
            Ok(UNNAMED.to_string())
        })
    }
}

// A choice as JSON.
fn decided(choice: &Choice) -> Value {
    let path =
        |named: &Option<PathBuf>| named.as_deref().map(Path::display).map(|at| at.to_string());
    match choice {
        Choice::Snapshot(at) => json!({ "surface": "snapshot", "path": at.display().to_string() }),
        Choice::Local(endpoint) => json!({ "surface": "local", "endpoint": endpoint }),
        Choice::Remote { address, named } => json!({
            "surface": "remote",
            "address": address,
            "certificate": path(&named.certificate),
            "key": path(&named.key),
            "anchors": path(&named.anchors),
            "kept": named.kept,
        }),
    }
}
