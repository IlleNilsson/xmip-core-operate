//! Text as it crosses the boundary, both ways: a `&str` lent to a surface
//! as the header's `xmip_str`, and a surface's `xmip_str` read as `&str`.
//! Every export of `xmip_operate.h` reads and lends text through these two,
//! a node's and a remote surface's alike; and the exports that take two
//! texts and answer one (sections 10 and 27) answer through [`answer`].
#![allow(unsafe_code)]

use abi::ffi::{Str, status};

use crate::ffi::rule::refuse;

/// `text` lent as the header's string: valid while `text` is.
pub(crate) fn borrow(text: &str) -> Str {
    Str {
        ptr: text.as_ptr(),
        len: text.len(),
    }
}

/// The scope a surface passed, as `&str`, or `None` if it is not UTF-8 —
/// which the header says to answer with `XMIP_E_MALFORMED`.
///
/// # Safety
/// `scope` must point at `scope.len` readable bytes, as the header requires.
pub(crate) unsafe fn scope_text<'a>(scope: Str) -> Option<&'a str> {
    if scope.ptr.is_null() {
        return Some("");
    }

    // SAFETY: the caller upholds the header's contract on `scope`.
    let bytes = unsafe { core::slice::from_raw_parts(scope.ptr, scope.len) };

    core::str::from_utf8(bytes).ok()
}

/// The bytes a surface lent as the header's `xmip_slice`; empty for null.
///
/// # Safety
/// `bytes` must point at `bytes.len` readable bytes, as the header requires.
pub(crate) unsafe fn slice_bytes<'a>(bytes: abi::ffi::Slice) -> &'a [u8] {
    if bytes.ptr.is_null() {
        return &[];
    }

    // SAFETY: the caller upholds the header's contract on `bytes`.
    unsafe { core::slice::from_raw_parts(bytes.ptr, bytes.len) }
}

/// Read both texts, answer, and write the answer or the refusal in the
/// header's text shape: its true length in `out_len` whether or not it fit.
/// Every export of sections 10 and 27 is this call alone, so it is their
/// outermost frame and holds their panic guard (specification section 6).
///
/// # Safety
/// `input` and `argument` point at their stated lengths of readable bytes;
/// `out` has room for `cap` bytes; `out_len` is writable.
pub(crate) unsafe fn answer(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
    answering: impl FnOnce(&str, &str) -> Result<String, String>,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: the caller upholds the header's contract on both texts:
        // their bytes are the caller's, alive for the call, and not freed.
        let (Some(input), Some(argument)) = (unsafe { scope_text(input) }, unsafe {
            scope_text(argument)
        }) else {
            return status::MALFORMED;
        };

        let (code, text) = match answering(input, argument) {
            Ok(answer) => (status::OK, answer),
            Err(refusal) => (status::INVALID, refusal),
        };

        // SAFETY: `out` has room for `cap` bytes and `out_len` is writable,
        // per the contract; the text shape a refusal takes is the answer's.
        unsafe { refuse(&text, out, cap, out_len) };
        code
    })
}
