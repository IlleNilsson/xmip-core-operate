//! `xmip_operate.h` section 27: a configuration document read, written and
//! one key of it set (ADR-0031, amendment 2026-11-17). The exports only:
//! what each does is `crate::document`'s, over `configure::document`. A
//! remote build carries them, so the phone reads its document too.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16): each reads the text a surface handed over
//! and writes its answer where it was told.
#![allow(unsafe_code)]

use abi::ffi::Str;

use crate::document;
use crate::ffi::text::answer;

/// `xmip_document_read_v1`: [`document::read`].
///
/// # Safety
/// As the header states: both texts readable, `out` room for `cap` bytes,
/// `out_len` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_document_read_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |text, _| {
            document::read(text)
        })
    }
}

/// `xmip_document_write_v1`: [`document::write`].
///
/// # Safety
/// As [`xmip_document_read_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_document_write_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |json, _| {
            document::write(json)
        })
    }
}

/// `xmip_document_set_v1`: [`document::set`].
///
/// # Safety
/// As [`xmip_document_read_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_document_set_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe { answer(input, argument, out, cap, out_len, document::set) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::text::borrow;
    use abi::ffi::status;

    #[test]
    fn every_export_has_the_shape_the_binding_declares() {
        let _: abi::operate::document::DocumentFn = xmip_document_read_v1;
        let _: abi::operate::document::DocumentFn = xmip_document_write_v1;
        let _: abi::operate::document::DocumentFn = xmip_document_set_v1;
    }

    fn called(
        export: abi::operate::document::DocumentFn,
        input: &str,
        argument: &str,
    ) -> (i32, String) {
        let mut out = vec![0_u8; 1024];
        let mut len = 0_usize;

        // SAFETY: two borrowed strings, a buffer of its stated room and a
        // writable length, as the header requires.
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

    #[test]
    fn a_document_crosses_as_json_and_back_and_a_refusal_is_invalid() {
        let (code, json) = called(xmip_document_read_v1, "[a]\nb = 1\n", "");
        assert_eq!((code, json.as_str()), (status::OK, r#"{"a":{"b":1}}"#));

        let (code, text) = called(xmip_document_write_v1, &json, "");
        assert_eq!((code, text.as_str()), (status::OK, "[a]\nb = 1\n"));

        let (code, text) = called(
            xmip_document_set_v1,
            "[a]\nb = 1\n",
            r#"{"table":["a"],"key":"b","value":2}"#,
        );
        assert_eq!((code, text.as_str()), (status::OK, "[a]\nb = 2\n"));

        let (code, refusal) = called(xmip_document_read_v1, "[a", "");
        assert_eq!(code, status::INVALID);
        assert!(refusal.starts_with(configure::PARSE_FAILED), "{refusal}");
    }
}
