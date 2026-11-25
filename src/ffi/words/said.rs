//! The fixed words, lent for the life of the library: a phrase, the word an
//! act crosses as, what came of an act, what a status code means and the
//! two boundaries this build speaks.
#![allow(unsafe_code)]

use std::sync::OnceLock;

use abi::boundaries::Boundaries;
use abi::ffi::{Str, status};
use abi::operate::words::{self, phrase};
use abi::status_meaning::StatusMeaning;
use observe::Act;

use crate::ffi::text::borrow;
use crate::surface::acted::ActOutcome;
use crate::surface::english;

/// `xmip_phrase_v1`: the phrase `which` names (`XMIP_PHRASE_*`).
///
/// # Safety
/// `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_phrase_v1(which: i32, out: *mut Str) -> i32 {
    abi::guarded(status::PANIC, || {
        let said = match which {
            phrase::NOTHING_RECORDED => english::mood_or_nothing(None),
            phrase::NO_AUDIT_DIRECTORY => english::NO_AUDIT_DIRECTORY,
            phrase::NO_WORK_PROCESS => english::NO_WORK_PROCESS,
            _ => return status::INVALID,
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(borrow(said)) };
        status::OK
    })
}

/// `xmip_act_word_v1`: the word the act `which` (`XMIP_ACT_*`, in
/// `observe::Act`'s order) crosses to a node as.
///
/// # Safety
/// `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_act_word_v1(which: i32, out: *mut Str) -> i32 {
    abi::guarded(status::PANIC, || {
        let named = usize::try_from(which)
            .ok()
            .and_then(|at| Act::ALL.get(at).copied());
        let Some(named) = named else {
            return status::INVALID;
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(borrow(named.word())) };
        status::OK
    })
}

/// `xmip_act_outcome_v1`: what came of an act whose export answered
/// `code` (`XMIP_ACTED_*`), and its word.
///
/// # Safety
/// `out` and `word` are writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_act_outcome_v1(code: i32, out: *mut i32, word: *mut Str) -> i32 {
    abi::guarded(status::PANIC, || {
        let outcome = ActOutcome::of_status(code);
        let number = match outcome {
            ActOutcome::Taken => words::outcome::TAKEN,
            ActOutcome::Asked => words::outcome::ASKED,
            ActOutcome::Refused => words::outcome::REFUSED,
        };
        // SAFETY: both are writable per the contract.
        unsafe {
            out.write(number);
            word.write(borrow(outcome.word()));
        }
        status::OK
    })
}

/// `xmip_status_meaning_v1`: what `code` means, `abi::status_meaning`'s.
///
/// # Safety
/// `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_status_meaning_v1(code: i32, out: *mut words::StatusMeaning) -> i32 {
    abi::guarded(status::PANIC, || {
        let meaning = StatusMeaning::of(code);
        let said = words::StatusMeaning {
            known: u8::from(meaning.known),
            name: borrow(meaning.name),
            meaning: borrow(meaning.meaning),
            retryable: u8::from(meaning.retryable),
            terminal: u8::from(meaning.terminal),
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(said) };
        status::OK
    })
}

/// `xmip_boundaries_v1`: the two boundaries this build speaks,
/// `abi::boundaries`'s, said once a process.
///
/// # Safety
/// `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_boundaries_v1(out: *mut words::Boundaries) -> i32 {
    static CURRENT: OnceLock<Boundaries> = OnceLock::new();

    abi::guarded(status::PANIC, || {
        let current = CURRENT.get_or_init(Boundaries::current);
        let said = words::Boundaries {
            module_version: current.module_version,
            module_entrypoint: borrow(current.module_entrypoint),
            module_library_file_name: borrow(&current.module_library_file_name),
            operate_version: current.operate_version,
            operate_entrypoint: borrow(current.operate_entrypoint),
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { out.write(said) };
        status::OK
    })
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use abi::operate::words::act;

    pub(crate) fn text(said: Str) -> String {
        // SAFETY: the export lent `len` readable bytes.
        let bytes = unsafe { core::slice::from_raw_parts(said.ptr, said.len) };
        String::from_utf8(bytes.to_vec()).expect("UTF-8")
    }

    #[test]
    fn the_exports_have_the_shapes_the_binding_declares() {
        let _: words::WordFn = xmip_phrase_v1;
        let _: words::WordFn = xmip_act_word_v1;
        let _: words::ActOutcomeFn = xmip_act_outcome_v1;
        let _: words::StatusMeaningFn = xmip_status_meaning_v1;
        let _: words::BoundariesFn = xmip_boundaries_v1;
    }

    #[test]
    fn a_phrase_an_act_and_an_outcome_are_the_rules_words() {
        let mut said = Str::empty();

        // SAFETY: a writable string.
        assert_eq!(
            unsafe { xmip_phrase_v1(phrase::NOTHING_RECORDED, &raw mut said) },
            0
        );
        assert_eq!(text(said), "nothing recorded");
        // SAFETY: as above.
        assert_eq!(
            unsafe { xmip_phrase_v1(99, &raw mut said) },
            status::INVALID
        );
        // SAFETY: as above.
        assert_eq!(unsafe { xmip_act_word_v1(act::DISMISS, &raw mut said) }, 0);
        assert_eq!(text(said), "dismiss");

        let mut outcome = -1;
        // SAFETY: a writable number and string.
        unsafe { xmip_act_outcome_v1(status::AGAIN, &raw mut outcome, &raw mut said) };
        assert_eq!(
            (outcome, text(said)),
            (words::outcome::ASKED, "asked".to_string())
        );
        // SAFETY: as above.
        unsafe { xmip_act_outcome_v1(status::AUTH, &raw mut outcome, &raw mut said) };
        assert_eq!(outcome, words::outcome::REFUSED);
    }

    #[test]
    fn a_status_and_the_boundaries_are_abis() {
        let mut meaning = words::StatusMeaning {
            known: 9,
            name: Str::empty(),
            meaning: Str::empty(),
            retryable: 9,
            terminal: 9,
        };
        // SAFETY: a writable record.
        unsafe { xmip_status_meaning_v1(status::TIMEOUT, &raw mut meaning) };
        assert_eq!(
            (meaning.known, meaning.retryable, meaning.terminal),
            (1, 1, 0)
        );
        assert_eq!(text(meaning.name), "Timeout");
        // SAFETY: as above.
        unsafe { xmip_status_meaning_v1(4711, &raw mut meaning) };
        assert_eq!(
            (meaning.known, text(meaning.name)),
            (0, "unknown".to_string())
        );

        let mut boundaries = words::Boundaries {
            module_version: 0,
            module_entrypoint: Str::empty(),
            module_library_file_name: Str::empty(),
            operate_version: 0,
            operate_entrypoint: Str::empty(),
        };
        // SAFETY: a writable record.
        unsafe { xmip_boundaries_v1(&raw mut boundaries) };
        assert_eq!(
            text(boundaries.operate_entrypoint),
            abi::operate::XMIP_OPERATE_ENTRYPOINT
        );
        assert!(text(boundaries.module_library_file_name).contains("xmip_core_transport_file"));
    }
}
