//! What a scope tree answers besides a scope: a list of scopes or names, a
//! scope's label, the worst record on a stage, one figure as a measurement,
//! every node's declaration, and what a pattern selects (section 28.1).

use abi::ffi::{Str, status};
use abi::operate::Measurement;
use abi::operate::scope_index::{ScopeIndex as Handle, list};
use serde_json::{Value, json};

use super::held;
use crate::ffi::rule::{fill, refuse};
use crate::ffi::text::{borrow, scope_text};
use crate::surface::scope_selection::ScopeSelection;
use crate::wire::{from_wire_counted, wire_counted};

/// A list of the index's scopes or names, borrowed, in the fill shape:
/// where a stage begins, the names beneath the root, or what a pattern
/// matches.
///
/// # Safety
/// `index` is a live handle; `argument` points at its stated length; `out`
/// has room for `cap` entries; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_list_v1(
    index: *const Handle,
    which: u32,
    argument: Str,
    out: *mut Str,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out_len.is_null()) else {
            return status::INVALID;
        };
        // SAFETY: `argument` points at its stated length per the contract.
        let Some(argument) = (unsafe { scope_text(argument) }) else {
            return status::MALFORMED;
        };
        let listed = match which {
            list::STAGE_SCOPES => index.stage_scopes_borrowed(argument),
            list::NODES => index.nodes_borrowed(),
            list::MATCHING => index.matching_borrowed(argument),
            _ => return status::INVALID,
        };
        // SAFETY: `out` has room for `cap` entries; `out_len` is writable.
        unsafe { fill(listed.into_iter(), out, cap, out_len) };
        status::OK
    })
}

/// What a person reads a scope as, borrowed from the index, static, or from
/// `scope` itself.
///
/// # Safety
/// `index` is a live handle; `scope` points at its stated length; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_label_v1(
    index: *const Handle,
    scope: Str,
    out: *mut Str,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out.is_null()) else {
            return status::INVALID;
        };
        // SAFETY: `scope` points at its stated length per the contract.
        let Some(scope) = (unsafe { scope_text(scope) }) else {
            return status::MALFORMED;
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(borrow(index.label_borrowed(scope))) };
        status::OK
    })
}

/// The place of the worst record on a stage, -1 for none.
///
/// # Safety
/// `index` is a live handle; `stage` points at its stated length;
/// `out_place` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_worst_at_stage_v1(
    index: *const Handle,
    stage: Str,
    out_place: *mut i64,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out_place.is_null()) else {
            return status::INVALID;
        };
        // SAFETY: `stage` points at its stated length per the contract.
        let Some(stage) = (unsafe { scope_text(stage) }) else {
            return status::MALFORMED;
        };
        let place = index.worst_at_stage_placed(stage).map_or(-1, i64::from);
        // SAFETY: `out_place` is writable per the contract.
        unsafe { *out_place = place };
        status::OK
    })
}

/// One figure at a scope as a measurement, its scope borrowed from `scope`.
///
/// # Safety
/// `index` is a live handle; `scope` points at its stated length; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_measure_v1(
    index: *const Handle,
    scope: Str,
    counted: i32,
    now_unix_nanos: i64,
    out: *mut Measurement,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out.is_null()) else {
            return status::INVALID;
        };
        let Some(kind) = from_wire_counted(counted) else {
            return status::INVALID;
        };
        // SAFETY: `scope` points at its stated length per the contract.
        let Some(text) = (unsafe { scope_text(scope) }) else {
            return status::MALFORMED;
        };
        let Some(count) = index.measure(text, kind, now_unix_nanos) else {
            return status::NOT_FOUND;
        };
        let measured = Measurement {
            scope,
            counted: wire_counted(count.counted),
            value: count.value,
            window_start_unix_nanos: count.window_start_unix_nanos,
            window_end_unix_nanos: count.window_end_unix_nanos,
            observed_unix_nanos: count.observed_unix_nanos,
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(measured) };
        status::OK
    })
}

/// What each node declared, as a JSON array by node.
///
/// # Safety
/// `index` is a live handle; `out` has room for `cap` bytes; `out_len` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_capabilities_v1(
    index: *const Handle,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out_len.is_null()) else {
            return status::INVALID;
        };
        let declared: Vec<Value> = index
            .capabilities()
            .into_iter()
            .map(|said| {
                json!({
                    "node": said.node,
                    "roles": said.roles,
                    "online": said.online,
                    "published": said.published,
                    "evidence": said.evidence,
                    "refusal": said.refusal,
                })
            })
            .collect();
        // SAFETY: `out` has room for `cap` bytes; `out_len` is writable.
        unsafe { refuse(&Value::Array(declared).to_string(), out, cap, out_len) };
        status::OK
    })
}

/// What a pattern selects among the index's scopes, as JSON; the refusal,
/// naming `source`, of a wildcard that matches nothing.
///
/// # Safety
/// `index` is a live handle; `pattern` and `source` point at their stated
/// lengths; `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_select_v1(
    index: *const Handle,
    pattern: Str,
    source: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out_len.is_null()) else {
            return status::INVALID;
        };
        // SAFETY: both point at their stated lengths per the contract.
        let (Some(pattern), Some(source)) = (unsafe { (scope_text(pattern), scope_text(source)) })
        else {
            return status::MALFORMED;
        };
        let (said, code) = match ScopeSelection::over(index, pattern, source) {
            Ok(chosen) => (
                json!({
                    "argument": chosen.argument,
                    "patterned": chosen.patterned,
                    "scopes": chosen.scopes,
                })
                .to_string(),
                status::OK,
            ),
            Err(refusal) => (refusal, status::NOT_FOUND),
        };
        // SAFETY: `out` has room for `cap` bytes; `out_len` is writable.
        unsafe { refuse(&said, out, cap, out_len) };
        code
    })
}
