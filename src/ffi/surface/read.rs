//! What a surface read, handed across where it changed: its head as JSON —
//! which reading it is, where a drill starts and who published — and the
//! publication itself, lent as section 8 lends one, with its scope tree as
//! section 28.1 holds one, each the caller's to free. Nothing is read a
//! second time: the publication handed is the one the surface read.

use std::sync::Arc;

use abi::ffi::status;
use abi::operate::publication::Publication as PublicationHandle;
use abi::operate::scope_index::ScopeIndex as IndexHandle;
use abi::operate::surface::Surface as Handle;
use serde_json::json;
use xcore::lock::Locked;

use super::held;
use crate::ffi::publication::Read;
use crate::ffi::rule::refuse;

/// The surface's reading, where it is another than `seen` names: its head as
/// JSON in `out` — `reading`, a number naming this reading among those the
/// surface handed; `changed`, true; `revision`; `root`, where a drill
/// starts; `said`, who published in its own words — and its publication and
/// scope tree in `out_publication` and `out_index`. Where it is the one
/// `seen` names, nothing: `out_len` 0 and both null, no JSON made.
/// `XMIP_E_TRUNCATED`, nothing handed, where the head does not fit: ask
/// again with room for `out_len`.
///
/// # Safety
/// `surface` is a live handle; `out_publication` and `out_index` are
/// writable; `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_read_v1(
    surface: *const Handle,
    seen: u64,
    out_publication: *mut *mut PublicationHandle,
    out_index: *mut *mut IndexHandle,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(held) = (unsafe { held(surface) }) else {
            return status::INVALID;
        };
        // SAFETY: both are writable per the contract.
        unsafe {
            *out_publication = core::ptr::null_mut();
            *out_index = core::ptr::null_mut();
        }

        let reading = held.surface.reading();
        let which = {
            let mut handed = held.handed.locked();
            let same = handed
                .0
                .as_ref()
                .is_some_and(|before| Arc::ptr_eq(before, &reading));
            if !same {
                handed.1 += 1;
                handed.0 = Some(Arc::clone(&reading));
            }
            handed.1
        };
        let changed = which != seen;
        // The same reading as the caller's: nothing to say, and nothing
        // written, so a question asked of an unchanged surface costs no JSON.
        if !changed {
            // SAFETY: `out_len` is writable per the contract.
            unsafe { *out_len = 0 };
            return status::OK;
        }
        let head = json!({
            "reading": which,
            "changed": changed,
            "revision": reading.index.revision,
            "root": reading.root,
            "said": reading.said,
        })
        .to_string();

        // SAFETY: per the contract above.
        unsafe { refuse(&head, out, cap, out_len) };
        if head.len() > cap {
            return status::TRUNCATED;
        }

        let read = Read::shared(Arc::clone(&reading.publication));
        // SAFETY: both are writable per the contract.
        unsafe {
            *out_publication = Box::into_raw(Box::new(read)).cast::<PublicationHandle>();
            *out_index = crate::ffi::scope_index::hand(Arc::clone(&reading.index));
        }
        status::OK
    })
}
