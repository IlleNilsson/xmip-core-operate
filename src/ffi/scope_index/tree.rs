//! The scope tree's composed rules a surface asks rather than keeps: a
//! scope's parent, its node's own scope and its name, and six figures
//! summed (section 28.1). Pure.

use abi::ffi::{Str, status};
use abi::operate::scope_index::{Figures as Wire, relative};

use crate::ffi::rule::refuse;
use crate::ffi::text::scope_text;
use crate::surface::figures::Figures;
use crate::surface::scope_tree;

/// A scope's parent, its node's own scope, or its name, written as UTF-8
/// into `out`, its true length in `out_len`.
///
/// # Safety
/// `scope` points at its stated length; `out` has room for `cap` bytes;
/// `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_scope_relative_v1(
    scope: Str,
    which: u32,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        if out_len.is_null() {
            return status::INVALID;
        }
        // SAFETY: `scope` points at its stated length per the contract.
        let Some(scope) = (unsafe { scope_text(scope) }) else {
            return status::MALFORMED;
        };
        let answer = match which {
            relative::PARENT => scope_tree::parent(scope),
            relative::NODE_SCOPE => scope_tree::node_scope(scope),
            relative::NAME => scope_tree::name(scope),
            _ => return status::INVALID,
        };
        // SAFETY: `out` has room for `cap` bytes; `out_len` is writable.
        unsafe { refuse(&answer, out, cap, out_len) };
        status::OK
    })
}

/// `len` figures summed into `out`: a kind no part published stays
/// unpublished, the newest observed wins.
///
/// # Safety
/// `parts` is null with a zero `len` or points at `len` figures; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_figures_sum_v1(
    parts: *const Wire,
    len: usize,
    out: *mut Wire,
) -> i32 {
    abi::guarded(status::PANIC, || {
        if out.is_null() || (parts.is_null() && len > 0) {
            return status::INVALID;
        }
        // SAFETY: per the contract above.
        let parts = unsafe { super::entries(parts, len) };
        let read: Vec<Figures> = parts.iter().map(figures).collect();
        let sum = Figures::sum("", &read);
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(wire(&sum)) };
        status::OK
    })
}

fn figures(wire: &Wire) -> Figures {
    let values = [
        wire.streams,
        wire.messages,
        wire.journeys,
        wire.bytes,
        wire.retrying,
        wire.failed,
    ];
    let mut read = Figures::none("");
    for (kind, value) in values.into_iter().enumerate() {
        if wire.published & (1 << kind) != 0 {
            read.values[kind] = Some(value);
        }
    }
    read.observed = (wire.observed != 0).then_some(wire.observed_unix_nanos);
    read
}

fn wire(figures: &Figures) -> Wire {
    let [streams, messages, journeys, bytes, retrying, failed] =
        figures.values.map(|value| value.unwrap_or(0));
    let published = figures
        .values
        .iter()
        .enumerate()
        .filter(|(_, value)| value.is_some())
        .fold(0u8, |bits, (kind, _)| bits | (1 << kind));

    Wire {
        streams,
        messages,
        journeys,
        bytes,
        retrying,
        failed,
        observed_unix_nanos: figures.observed.unwrap_or(0),
        published,
        observed: u8::from(figures.observed.is_some()),
    }
}
