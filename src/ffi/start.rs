//! `xmip_operate.h` section 6: starting and validating a node, from a
//! surface. The exports only — what starting and validating are is
//! `runtime::start`'s, each Location held to every technology's declaration,
//! carried once in the process (`crate::catalogue`; ADR-0064, amendment
//! 2026-11-26).
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16): each reads text a surface handed over and
//! writes where it was told.
#![allow(unsafe_code)]

use abi::ffi::{Str, status};

use crate::ffi::rule::hand;
use crate::ffi::text::scope_text;
use crate::operator::start_published;
use runtime::start::validate;

/// `xmip_start_v1`: [`runtime::start::start`] from a surface. Publishes whatever it found and
/// returns `XMIP_OK` when the node validated, `XMIP_E_INVALID` when it did
/// not, `XMIP_E_MALFORMED` when the path is not UTF-8. The snapshot says why
/// either way, so a surface reads the table rather than the status.
///
/// # Safety
/// `path` must point at `path.len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_start_v1(path: Str) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: `path` points at its stated length, per the contract.
        let Some(text) = (unsafe { scope_text(path) }) else {
            return status::MALFORMED;
        };

        crate::catalogue::carried();
        if start_published(text) {
            status::OK
        } else {
            status::INVALID
        }
    })
}

/// `xmip_validate_v1`: [`runtime::start::validate`] from a surface. The report is written into
/// `report` as UTF-8, one problem per line, and `out_len` is the true byte
/// length whether or not it fit — a surface that passed too small a buffer
/// asks again. `XMIP_OK` and `out_len` 0 means the configuration is good;
/// `XMIP_E_INVALID` with a report means it is not; `XMIP_E_MALFORMED` when the
/// configuration is not UTF-8.
///
/// # Safety
/// `configuration` points at its stated length of readable bytes; `report`
/// has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_validate_v1(
    configuration: Str,
    report: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: the caller upholds the header's contract on
        // `configuration`: its bytes are the caller's, alive for the call.
        let Some(text) = (unsafe { scope_text(configuration) }) else {
            return status::MALFORMED;
        };

        crate::catalogue::carried();
        let problems = validate(text);
        // SAFETY: `report` has room for `cap` bytes and `out_len` is
        // writable per the contract.
        unsafe { hand(problems.join("\n").as_bytes(), report, cap, out_len) };

        if problems.is_empty() {
            status::OK
        } else {
            status::INVALID
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_exports_have_the_shape_the_binding_declares() {
        // A signature that drifted from xmip-core-abi's fails to compile here,
        // and the language server calls through that same declaration.
        let _: abi::operate::StartFn = xmip_start_v1;
        let _: abi::operate::ValidateFn = xmip_validate_v1;
    }

    #[test]
    fn a_locations_settings_are_held_to_its_technologys_declaration_no_node_here_loaded() {
        let cluster = configure::fixture::test_cluster();
        let node = &cluster.node(0).name;
        let configuration = format!(
            "[service]\nname = \"xmip-{node}\"\ncluster_name = \"{}\"\nnode_name = \"{node}\"\n\n\
             [[send_locations]]\nname = \"billing-out\"\naddress = \"C:/out\"\n\
             [send_locations.settings]\nnamed = \"{{message}}.edi\"\n",
            cluster.name
        );
        let given = Str {
            ptr: configuration.as_ptr(),
            len: configuration.len(),
        };
        let mut report = vec![0u8; 4096];
        let mut needed = 0usize;

        // SAFETY: `given` borrows the configuration for the call, and
        // `report` has room for its length.
        let code =
            unsafe { xmip_validate_v1(given, report.as_mut_ptr(), report.len(), &raw mut needed) };

        let said = String::from_utf8_lossy(&report[..needed.min(report.len())]).to_string();
        assert_eq!(code, status::INVALID, "{said}");
        assert!(
            said.contains("xmip-core-transport-file has no setting \"named\""),
            "{said}"
        );
    }
}
