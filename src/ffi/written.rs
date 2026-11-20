//! `xmip_operate.h` sections 19, 20 and 23, what a surface asks written or
//! named for it rather than doing it itself: a person's request signed into
//! its chain's file (`xmip_operator_sign_file_v1`), an upgrade's scripts
//! written into a folder (`xmip_database_write_v1`), and the name a
//! cluster's own authority is made under (`xmip_authority_name_v1`). Each
//! is the rule the command line calls, `crate::certificate` and
//! `crate::database`; until 2026-11-17 each .NET surface wrote it again.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16): each reads the text a surface handed
//! over and writes its answer where it was told.
#![allow(unsafe_code)]

use std::path::Path;

use abi::ffi::Str;
use serde_json::{Value, json};

use crate::database::{self, Upgrade};
use crate::ffi::text::answer;

/// `xmip_operator_sign_file_v1`: [`crate::certificate::sign_request`] — the
/// request at the path `input` signed by the authority the node
/// configuration `argument`'s `configuration` names, its chain kept whole
/// at `argument`'s `chain` where that is a path, the chain, PEM, answered.
///
/// # Safety
/// As the header states: both texts readable, `out` room for `cap` bytes,
/// `out_len` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_operator_sign_file_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |request, asked| {
            let asked: Value =
                serde_json::from_str(asked).map_err(|unread| format!("not JSON: {unread}"))?;
            let configuration = asked["configuration"]
                .as_str()
                .ok_or("an argument names its configuration")?;
            let chain = asked["chain"].as_str().map(Path::new);
            crate::certificate::sign_request(Path::new(request), Path::new(configuration), chain)
        })
    }
}

/// `xmip_database_write_v1`: [`database::write`] — the scripts of the
/// upgrade `input`, as `xmip_database_upgrade_v1` answers one, written into
/// the folder `argument`, none over a file; the paths written answered as
/// a JSON array.
///
/// # Safety
/// As [`xmip_operator_sign_file_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_database_write_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |upgrade, folder| {
            let read: Value =
                serde_json::from_str(upgrade).map_err(|unread| format!("not JSON: {unread}"))?;
            let scripts = read["scripts"]
                .as_array()
                .ok_or("an upgrade carries its scripts")?
                .iter()
                .map(|script| {
                    let text = |name: &str| script[name].as_str().unwrap_or_default().to_string();
                    (text("name"), text("text"))
                })
                .collect();
            let version = read["version"]
                .as_u64()
                .and_then(|version| u32::try_from(version).ok())
                .unwrap_or(0);
            let written = database::write(&Upgrade { version, scripts }, Path::new(folder))?;
            let paths: Vec<String> = written
                .iter()
                .map(|path| path.display().to_string())
                .collect();
            Ok(json!(paths).to_string())
        })
    }
}

/// `xmip_authority_name_v1`: [`crate::certificate::authority_name`] — the
/// name the cluster `input` names makes its own authority under.
///
/// # Safety
/// As [`xmip_operator_sign_file_v1`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_authority_name_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |cluster, _| {
            Ok(crate::certificate::authority_name(cluster))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ffi::text::borrow;
    use abi::ffi::status;

    type TextFn = unsafe extern "C" fn(Str, Str, *mut u8, usize, *mut usize) -> i32;

    fn asked(export: TextFn, input: &str, argument: &str) -> (i32, String) {
        let mut out = vec![0u8; 4096];
        let mut len = 0usize;
        // SAFETY: both texts live for the call; `out` has room for its length.
        let code = unsafe {
            export(
                borrow(input),
                borrow(argument),
                out.as_mut_ptr(),
                out.len(),
                &raw mut len,
            )
        };
        out.truncate(len.min(out.len()));
        (code, String::from_utf8(out).expect("UTF-8"))
    }

    #[test]
    fn the_authority_is_named_for_its_cluster_and_a_request_not_there_is_said_so() {
        assert_eq!(
            asked(xmip_authority_name_v1, "c", ""),
            (status::OK, "Xmip cluster c authority".to_string())
        );
        let folder = file::temporary::Folder::new("operate-written-no-request");
        let request = folder.join("no-request.csr").display().to_string();
        let (code, said) = asked(
            xmip_operator_sign_file_v1,
            &request,
            r#"{"configuration":"node.xmip","chain":null}"#,
        );
        assert_eq!(
            (code, said),
            (status::INVALID, format!("No file at {request}."))
        );
    }

    #[test]
    fn scripts_are_written_once_and_never_over_a_file() {
        let place = file::temporary::Folder::new("operate-written");
        let upgrade = r#"{"version":2,"scripts":[{"name":"a.sql","text":"select 1;"}]}"#;
        let into = place.join("upgrade").display().to_string();

        let (code, paths) = asked(xmip_database_write_v1, upgrade, &into);
        assert_eq!(code, status::OK, "{paths}");
        assert!(paths.contains("a.sql"), "{paths}");
        let (again, refused) = asked(xmip_database_write_v1, upgrade, &into);
        assert_eq!(again, status::INVALID);
        assert!(refused.starts_with("REFUSED: "), "{refused}");
    }
}
