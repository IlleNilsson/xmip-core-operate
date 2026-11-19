//! `xmip_operate.h` section 26: a module library, probed (ADR-0012,
//! amendment 2026-11-17). The export only: the opening and the judgment are
//! the runtime loader's (`runtime::ffi::probed`), the one a node opens its
//! modules with, and every surface shows what this answers.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16): it reads the path a surface handed over
//! and writes its answer where it was told.
#![allow(unsafe_code)]

use abi::ffi::{Str, status};

use crate::ffi::rule::refuse;
use crate::ffi::text::scope_text;
use crate::module_probe;

/// `xmip_module_probe_v1`: `crate::module_probe::probe`, the one the command
/// line calls, answered as the header's JSON.
///
/// # Safety
/// As the header states: `library` readable for its length, `out` room for
/// `cap` bytes, `out_len` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_module_probe_v1(
    library: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: the caller upholds the header's contract on `library`:
        // its bytes are the caller's, alive for the call, and not freed.
        let Some(library) = (unsafe { scope_text(library) }) else {
            return status::MALFORMED;
        };

        let answer = module_probe::probe(library).to_json().to_string();

        // SAFETY: `out` has room for `cap` bytes and `out_len` is writable,
        // per the contract; the answer takes the text shape a refusal does.
        unsafe { refuse(&answer, out, cap, out_len) };
        status::OK
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::text::borrow;
    use serde_json::json;

    #[test]
    fn the_export_has_the_shape_the_binding_declares() {
        let _: abi::operate::module_probe::ModuleProbeFn = xmip_module_probe_v1;
    }

    fn asked(library: &str) -> serde_json::Value {
        let mut out = vec![0_u8; 4096];
        let mut len = 0_usize;

        // SAFETY: a borrowed string, a buffer of its stated room and a
        // writable length, as the header requires.
        let code = unsafe {
            xmip_module_probe_v1(borrow(library), out.as_mut_ptr(), out.len(), &raw mut len)
        };

        assert_eq!(code, status::OK);
        assert!(len <= out.len(), "the answer fits");
        serde_json::from_slice(&out[..len]).expect("the answer is JSON")
    }

    #[test]
    fn a_library_that_is_not_there_is_answered_unloadable_with_every_member() {
        let answer = asked("no_such_module_library.dll");

        assert_eq!(answer["library"], "no_such_module_library.dll");
        assert_eq!(answer["status"], status::NOT_FOUND);
        assert_eq!(answer["conforms"], false);
        assert_eq!(answer["provider"], "");
        assert_eq!(answer["abi_version"], 0);
        assert!(
            answer["unloadable"]
                .as_str()
                .is_some_and(|why| why.contains("would not open")),
            "{answer}"
        );
        assert_eq!(answer["log"], json!([]));
    }

    #[test]
    fn a_path_that_is_not_utf8_is_malformed() {
        let bytes = [0xff_u8, 0xfe];
        let library = Str {
            ptr: bytes.as_ptr(),
            len: bytes.len(),
        };
        let mut len = 0_usize;

        // SAFETY: two readable bytes; no room, and a writable length.
        let code = unsafe { xmip_module_probe_v1(library, std::ptr::null_mut(), 0, &raw mut len) };

        assert_eq!(code, status::MALFORMED);
    }
}
