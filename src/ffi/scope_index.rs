//! `xmip_operate.h` section 28.1: a publication's scope tree,
//! `operate::surface::scope_index`, built once and asked by scope, every
//! record answered by its place; what else the tree answers is
//! `scope_index/asked.rs`, and the scope tree's composed rules
//! `scope_index/tree.rs`.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16).
#![allow(unsafe_code)]

mod asked;
#[cfg(test)]
mod tests;
mod tree;

use std::sync::Arc;

use abi::ffi::{Str, status};
use abi::operate::publication::Publication as PublicationHandle;
use abi::operate::scope_index::{ScopeIndex as Handle, ScopeIndexFacts, ScopePlaced};
use abi::operate::{HealthEntry, Measurement};
use observe::{Count, HealthRecord};

use crate::ffi::publication::held as publication;
use crate::ffi::text::{borrow, scope_text};
use crate::surface::scope_index::{Placed, ScopeIndex};
use crate::wire::{from_wire_counted, from_wire_health, wire_health};

/// A scope tree handed across: one holder of the index, which a surface's
/// reading may hold too (section 28.2).
pub(crate) struct Held(pub(crate) Arc<ScopeIndex>);

/// `index` handed across as a handle the caller frees.
pub(crate) fn hand(index: Arc<ScopeIndex>) -> *mut Handle {
    Box::into_raw(Box::new(Held(index))).cast::<Handle>()
}

/// The scope tree behind a handle.
///
/// # Safety
/// `index` is null or a handle this section or section 28.2 handed out and
/// nobody freed.
pub(crate) unsafe fn held<'a>(index: *const Handle) -> Option<&'a ScopeIndex> {
    // SAFETY: per the contract above.
    unsafe { index.cast::<Held>().as_ref() }.map(|held| held.0.as_ref())
}

/// Release a scope tree's handle and everything borrowed from it.
///
/// # Safety
/// `index` is null or a handle this section or section 28.2 handed out and
/// nobody freed; nothing borrowed from it is used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_free_v1(index: *mut Handle) {
    if !index.is_null() {
        // SAFETY: per the contract above, the box this handle was made from.
        let held = unsafe { Box::from_raw(index.cast::<Held>()) };
        abi::guarded((), || drop(held));
    }
}

/// An index built from `records` and `counts`, each record keeping its
/// place, said to be revision `revision` of `source`.
///
/// # Safety
/// `records` and `counts` are null with a zero length or point at their
/// lengths of entries whose strings point at their stated lengths; `source`
/// likewise; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_build_v1(
    records: *const HealthEntry,
    records_len: usize,
    counts: *const Measurement,
    counts_len: usize,
    revision: u64,
    source: Str,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        if out.is_null()
            || (records.is_null() && records_len > 0)
            || (counts.is_null() && counts_len > 0)
        {
            return status::INVALID;
        }
        // SAFETY: per the contract above.
        let built = unsafe { read(records, records_len, counts, counts_len, source) };
        let (records, counts, source) = match built {
            Ok(read) => read,
            Err(code) => return code,
        };
        let index = ScopeIndex::build(records, &counts, revision, source);
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(Arc::new(index)) };
        status::OK
    })
}

/// The records, counts and source a build hands over, read; the status to
/// answer where one is not what the header says.
///
/// # Safety
/// As [`xmip_scope_index_build_v1`]'s contract, the nulls already refused.
unsafe fn read<'a>(
    records: *const HealthEntry,
    records_len: usize,
    counts: *const Measurement,
    counts_len: usize,
    source: Str,
) -> Result<(Vec<HealthRecord>, Vec<Count>, &'a str), i32> {
    // SAFETY: per the contract above.
    let (records, counts) = unsafe { (entries(records, records_len), entries(counts, counts_len)) };
    let mut read = Vec::with_capacity(records.len());

    for entry in records {
        // SAFETY: each entry's strings point at their stated lengths.
        let (scope, evidence) = unsafe { (scope_text(entry.scope), scope_text(entry.evidence)) };
        let (Some(scope), Some(evidence)) = (scope, evidence) else {
            return Err(status::MALFORMED);
        };
        let health = from_wire_health(entry.health).ok_or(status::INVALID)?;
        read.push(HealthRecord {
            scope: scope.to_string(),
            health,
            severity: entry.severity,
            evidence: evidence.to_string(),
            observed_unix_nanos: entry.observed_unix_nanos,
        });
    }

    let mut counted = Vec::with_capacity(counts.len());

    for count in counts {
        // SAFETY: each count's scope points at its stated length.
        let scope = unsafe { scope_text(count.scope) }.ok_or(status::MALFORMED)?;
        counted.push(Count {
            scope: scope.to_string(),
            counted: from_wire_counted(count.counted).ok_or(status::INVALID)?,
            value: count.value,
            window_start_unix_nanos: count.window_start_unix_nanos,
            window_end_unix_nanos: count.window_end_unix_nanos,
            observed_unix_nanos: count.observed_unix_nanos,
        });
    }

    // SAFETY: `source` points at its stated length per the contract.
    let source = unsafe { scope_text(source) }.ok_or(status::MALFORMED)?;
    Ok((read, counted, source))
}

/// `len` entries at `at`, borrowed; none for a zero length.
///
/// # Safety
/// `at` points at `len` entries where `len` is not zero.
unsafe fn entries<'a, T>(at: *const T, len: usize) -> &'a [T] {
    if len == 0 {
        return &[];
    }
    // SAFETY: per the contract above.
    unsafe { core::slice::from_raw_parts(at, len) }
}

/// An index built from a read publication's records and counts, each record
/// at the place `xmip_publication_records_v1` hands it out.
///
/// # Safety
/// `publication` is a live publication handle; `source` points at its
/// stated length; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_of_publication_v1(
    publication_handle: *const PublicationHandle,
    revision: u64,
    source: Str,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(read), false) = (unsafe { publication(publication_handle) }, out.is_null())
        else {
            return status::INVALID;
        };
        // SAFETY: `source` points at its stated length per the contract.
        let Some(source) = (unsafe { scope_text(source) }) else {
            return status::MALFORMED;
        };
        let published = &read.publication;
        let index = ScopeIndex::build(
            published.records.clone(),
            &published.counts,
            revision,
            source,
        );
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(Arc::new(index)) };
        status::OK
    })
}

/// What an index is: its revision, its records and scopes, when its newest
/// figure was observed, and its source, borrowed.
///
/// # Safety
/// `index` is a live handle; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_facts_v1(
    index: *const Handle,
    out: *mut ScopeIndexFacts,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out.is_null()) else {
            return status::INVALID;
        };
        let facts = ScopeIndexFacts {
            revision: index.revision,
            leaves: index.leaves(),
            scopes: index.scopes(),
            observed_unix_nanos: index.observed.unwrap_or(0),
            observed: u8::from(index.observed.is_some()),
            source: borrow(&index.source),
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(facts) };
        status::OK
    })
}

/// One scope as the index holds it; `XMIP_E_NOT_FOUND` where it holds none.
///
/// # Safety
/// `index` is a live handle; `scope` points at its stated length; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_placed_v1(
    index: *const Handle,
    scope: Str,
    out: *mut ScopePlaced,
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
        match index.placed(scope) {
            Some(placed) => {
                // SAFETY: `out` is writable per the contract.
                unsafe { out.write(wire(&placed)) };
                status::OK
            }
            None => status::NOT_FOUND,
        }
    })
}

/// The scopes directly beneath one, worst first, in the fill shape.
///
/// # Safety
/// `index` is a live handle; `scope` points at its stated length; `out` has
/// room for `cap` entries; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_index_children_v1(
    index: *const Handle,
    scope: Str,
    out: *mut ScopePlaced,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let (Some(index), false) = (unsafe { held(index) }, out_len.is_null()) else {
            return status::INVALID;
        };
        // SAFETY: `scope` points at its stated length per the contract.
        let Some(scope) = (unsafe { scope_text(scope) }) else {
            return status::MALFORMED;
        };
        let children = index.children_placed(scope);
        for (at, child) in children.iter().enumerate().take(cap) {
            if !out.is_null() {
                // SAFETY: `at < cap` and `out` has room for `cap` entries.
                unsafe { out.add(at).write(wire(child)) };
            }
        }
        // SAFETY: `out_len` is writable per the contract.
        unsafe { *out_len = children.len() };
        status::OK
    })
}

/// A placed scope as the header lays it out, borrowed from the index.
fn wire(placed: &Placed<'_>) -> ScopePlaced {
    let mood = |health: Option<observe::Health>| health.map_or(-1, wire_health);
    let [streams, messages, journeys, bytes, retrying, failed] =
        placed.sums.map(|sum| sum.unwrap_or(0));
    let published = placed
        .sums
        .iter()
        .enumerate()
        .filter(|(_, sum)| sum.is_some())
        .fold(0u8, |bits, (kind, _)| bits | (1 << kind));

    ScopePlaced {
        scope: borrow(placed.scope),
        label: borrow(placed.label),
        leaves: placed.leaves.as_ptr(),
        leaves_len: placed.leaves.len(),
        own: placed.own.map_or(-1, i64::from),
        mood: mood(placed.mood),
        row_mood: mood(placed.row_mood),
        is_leaf: u8::from(placed.is_leaf),
        is_container: u8::from(placed.is_container),
        begins_stage: u8::from(placed.begins_stage),
        published,
        stage: borrow(placed.stage),
        streams,
        messages,
        journeys,
        bytes,
        retrying,
        failed,
    }
}
