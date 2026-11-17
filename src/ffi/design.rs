//! `xmip_operate.h` section 10: `cluster.xmip` read, edited and
//! sliced (ADR-0064, amendment 2026-10-03; ADR-0031, amendment 2026-10-05).
//! The exports only: what each does is `crate::design`'s.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16): each reads the text a surface handed over
//! and writes its answer where it was told.
#![allow(unsafe_code)]

use abi::ffi::Str;

use crate::design;
use crate::ffi::text::answer;

/// `xmip_cluster_views_v1`: [`design::views`].
///
/// # Safety
/// As the header states: both texts readable, `out` room for `cap` bytes,
/// `out_len` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_cluster_views_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |text, _| {
            design::views(text)
        })
    }
}

/// `xmip_filter_structure_v1`: [`design::filter_structure`].
///
/// # Safety
/// As [`xmip_cluster_views_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_filter_structure_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |text, _| {
            design::filter_structure(text)
        })
    }
}

/// `xmip_filter_text_v1`: [`design::filter_text`].
///
/// # Safety
/// As [`xmip_cluster_views_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_filter_text_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |text, _| {
            design::filter_text(text)
        })
    }
}

/// `xmip_cluster_edit_v1`: [`design::edit`].
///
/// # Safety
/// As [`xmip_cluster_views_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_cluster_edit_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe { answer(input, argument, out, cap, out_len, design::edit) }
}

/// `xmip_cluster_slices_v1`: [`design::slices`].
///
/// # Safety
/// As [`xmip_cluster_views_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_cluster_slices_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe { answer(input, argument, out, cap, out_len, design::slices) }
}

/// `xmip_cluster_signed_slice_v1`: [`design::signed_slice`].
///
/// # Safety
/// As [`xmip_cluster_views_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_cluster_signed_slice_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe { answer(input, argument, out, cap, out_len, design::signed_slice) }
}
#[cfg(test)]
mod tests {
    use super::*;
    use abi::ffi::status;

    #[test]
    fn every_export_has_the_shape_the_binding_declares() {
        // A signature that drifted from xmip-core-abi's fails to compile here,
        // and the language server calls through that same declaration.
        let _: abi::operate::design::DesignFn = xmip_cluster_views_v1;
        let _: abi::operate::design::DesignFn = xmip_filter_structure_v1;
        let _: abi::operate::design::DesignFn = xmip_filter_text_v1;
        let _: abi::operate::design::DesignFn = xmip_cluster_edit_v1;
        let _: abi::operate::design::DesignFn = xmip_cluster_slices_v1;
        let _: abi::operate::design::DesignFn = xmip_cluster_signed_slice_v1;
    }

    fn text(value: &str) -> Str {
        Str {
            ptr: value.as_ptr(),
            len: value.len(),
        }
    }

    #[test]
    fn an_answer_is_written_whole_and_a_refusal_says_why() {
        let filter = "exists Urgent";
        let mut needed = 0usize;

        // SAFETY: a null buffer of capacity 0 asks for the length only.
        let code = unsafe {
            xmip_filter_structure_v1(
                text(filter),
                text(""),
                std::ptr::null_mut(),
                0,
                &raw mut needed,
            )
        };
        assert_eq!(code, status::OK);

        let mut buffer = vec![0u8; needed];
        // SAFETY: `buffer` has room for `needed` bytes.
        let code = unsafe {
            xmip_filter_structure_v1(
                text(filter),
                text(""),
                buffer.as_mut_ptr(),
                buffer.len(),
                &raw mut needed,
            )
        };
        assert_eq!(code, status::OK);
        let said = String::from_utf8(buffer).expect("UTF-8");
        assert!(said.contains("\"operator\":\"exists\""), "{said}");

        let mut refusal = vec![0u8; 256];
        // SAFETY: `refusal` has room for 256 bytes.
        let code = unsafe {
            xmip_filter_structure_v1(
                text("not a filter"),
                text(""),
                refusal.as_mut_ptr(),
                refusal.len(),
                &raw mut needed,
            )
        };
        assert_eq!(code, status::INVALID);
        assert!(needed > 0);
    }

    /// An export whose answering panics, as a defect in what it forwards
    /// would: shaped as every export of this file is.
    unsafe extern "C" fn panicking(
        input: Str,
        argument: Str,
        out: *mut u8,
        cap: usize,
        out_len: *mut usize,
    ) -> i32 {
        // SAFETY: the caller's contract is passed on whole.
        unsafe {
            answer(input, argument, out, cap, out_len, |_, _| {
                panic!("a defect in what the export forwards")
            })
        }
    }

    #[test]
    fn a_panic_in_an_export_answers_panic_across_the_c_boundary_and_never_aborts() {
        // Called through the header's own function type, as a surface calls:
        // without the guard the panic reaches the `extern "C"` frame and the
        // whole test process aborts.
        let export: abi::operate::design::DesignFn = panicking;
        let mut needed = 7usize;
        // SAFETY: a null buffer of capacity 0, and a writable length.
        let code = unsafe {
            export(
                text("x"),
                text(""),
                std::ptr::null_mut(),
                0,
                &raw mut needed,
            )
        };
        assert_eq!(code, status::PANIC);
        assert!(status::is_terminal(code), "section 6: the instance is done");
    }
}
