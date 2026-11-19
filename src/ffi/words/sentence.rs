//! What a surface says, written into the caller's buffer, its true length
//! in `out_len` whether or not it fit: the sentences about a scope, an
//! act's outcome said, an age, a figure and the six of them, a value and a
//! time to the second, who acts, and a validation's verdict.
#![allow(unsafe_code)]

use abi::ffi::{Str, status};
use abi::operate::words::{outcome, sentence, text};

use crate::ffi::rule::refuse;
use crate::ffi::text::{answer, scope_text};
use crate::surface::acted::{self, ActOutcome};
use crate::surface::configuration_verdict::ConfigurationVerdict;
use crate::surface::english;
use crate::surface::figures::Figures;

/// The `len` strings at `strings`, owned, or `None` where one is not UTF-8.
///
/// # Safety
/// `strings` is null with `len` zero, or points at `len` readable strings.
unsafe fn texts(strings: *const Str, len: usize) -> Option<Vec<String>> {
    if len == 0 {
        return Some(Vec::new());
    }
    // SAFETY: per the contract above.
    let strings = unsafe { core::slice::from_raw_parts(strings, len) };
    strings
        .iter()
        // SAFETY: each string points at its stated length of readable bytes.
        .map(|text| unsafe { scope_text(*text) }.map(str::to_string))
        .collect()
}

/// Write `said` and answer `XMIP_OK`.
///
/// # Safety
/// `out` has room for `cap` bytes; `out_len` is writable.
unsafe fn written(said: &str, out: *mut u8, cap: usize, out_len: *mut usize) -> i32 {
    // SAFETY: per the contract above.
    unsafe { refuse(said, out, cap, out_len) };
    status::OK
}

/// `xmip_sentence_v1`: the sentence `which` (`XMIP_SENTENCE_*`) says of
/// `scope` on the surface `source`.
///
/// # Safety
/// `scope` and `source` point at their stated lengths of readable bytes;
/// `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_sentence_v1(
    which: i32,
    scope: Str,
    source: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(
            scope,
            source,
            out,
            cap,
            out_len,
            |scope, source| match which {
                sentence::NOTHING_AT => Ok(english::nothing_at(scope, source)),
                sentence::NOTHING_BENEATH => Ok(english::nothing_beneath(scope, source)),
                sentence::NOTHING_MEASURED => Ok(english::nothing_measured(scope, source)),
                _ => Err(format!("{which} is no sentence of section 28.3")),
            },
        )
    }
}

/// `xmip_act_said_v1`: an act's outcome (`XMIP_ACTED_*`) and the node's
/// words as a person reads them: `OK.`, `ASKED.` or `REFUSED:`, then them.
///
/// # Safety
/// `result` points at its stated length of readable bytes; `out` has room
/// for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_act_said_v1(
    which: i32,
    result: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    let came = match which {
        outcome::TAKEN => ActOutcome::Taken,
        outcome::ASKED => ActOutcome::Asked,
        outcome::REFUSED => ActOutcome::Refused,
        _ => return status::INVALID,
    };
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(result, Str::empty(), out, cap, out_len, |result, _| {
            Ok(came.said(result))
        })
    }
}

/// `xmip_age_v1`: how long before `now` `observed` was, both Unix
/// nanoseconds: `12s ago`, `3m ago`, `2h ago`.
///
/// # Safety
/// `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_age_v1(
    observed: i64,
    now: i64,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        unsafe { written(&english::age(observed, now), out, cap, out_len) }
    })
}

/// `xmip_figure_v1`: `value` with its thousands grouped, or a dash where
/// `has` says none was published.
///
/// # Safety
/// `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_figure_v1(
    has: u8,
    value: u64,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        let figure = english::figure((has != 0).then_some(value));
        // SAFETY: per the contract above.
        unsafe { written(&figure, out, cap, out_len) }
    })
}

/// `xmip_figures_line_v1`: the six figures — Streams, Messages, Journeys,
/// bytes, Retrying, Failed — in one line, each a dash where `has` says none
/// was published.
///
/// # Safety
/// `has` and `values` each point at six readable entries; `out` has room
/// for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_figures_line_v1(
    has: *const u8,
    values: *const u64,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        if has.is_null() || values.is_null() {
            return status::INVALID;
        }
        let mut figures = Figures::none("");
        for at in 0..6 {
            // SAFETY: six readable entries each, per the contract.
            let (has, value) = unsafe { (*has.add(at), *values.add(at)) };
            figures.values[at] = (has != 0).then_some(value);
        }
        // SAFETY: per the contract above.
        unsafe { written(&english::figures(&figures), out, cap, out_len) }
    })
}

/// `xmip_text_v1`: `value` as a surface shows it (`XMIP_TEXT_*`): itself or
/// a dash where it is blank; or an RFC 3339 time without its fraction.
///
/// # Safety
/// `value` points at its stated length of readable bytes; `out` has room
/// for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_text_v1(
    which: i32,
    value: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(
            value,
            Str::empty(),
            out,
            cap,
            out_len,
            |value, _| match which {
                text::VALUE => Ok(english::value(value).to_string()),
                text::TO_THE_SECOND => Ok(english::to_the_second(value)),
                _ => Err(format!("{which} is no text of section 28.3")),
            },
        )
    }
}

/// `xmip_who_v1`: who acts — `stated`, trimmed, where it names someone,
/// else `user`, the account the surface runs as.
///
/// # Safety
/// As section 10's shape: both texts readable for their lengths; `out` has
/// room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_who_v1(
    stated: Str,
    user: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(stated, user, out, cap, out_len, |stated, user| {
            Ok(acted::who(Some(stated), user))
        })
    }
}

/// `xmip_validated_v1`: the sentence the runtime's verdict on `path` says,
/// its status and the problems it listed.
///
/// # Safety
/// `path` points at its stated length of readable bytes; `problems` holds
/// `problems_len` readable strings; `out` has room for `cap` bytes;
/// `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_validated_v1(
    path: Str,
    code: i32,
    problems: *const Str,
    problems_len: usize,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        if problems.is_null() && problems_len > 0 {
            return status::INVALID;
        }
        // SAFETY: the caller upholds the header's contract on every string.
        let read = unsafe { (scope_text(path), texts(problems, problems_len)) };
        let (Some(path), Some(problems)) = read else {
            return status::MALFORMED;
        };
        let verdict = ConfigurationVerdict::validated(path, code, problems);
        // SAFETY: per the contract above.
        unsafe { written(&verdict.said, out, cap, out_len) }
    })
}
