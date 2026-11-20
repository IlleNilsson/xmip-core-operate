//! `xmip_operate.h` section 23: the cluster's authority kept in a folder,
//! and a node's certificate carried across an air gap (ADR-0034, amendment
//! 2026-10-22). One export, an act and its arguments as JSON, the answer as
//! JSON: every act is `crate::authority::act`'s, the one the command line
//! calls, and this reads what crossed and writes the answer. A node's build
//! only, as section 22.
#![allow(unsafe_code)]

use abi::ffi::{Str, status};
use serde_json::Value;

use crate::authority;
use crate::ffi::rule::refuse;
use crate::ffi::text::scope_text;

/// The act `act` on the arguments `arguments`, a JSON object, answered as
/// JSON in `out`; `XMIP_E_INVALID` with the refusal in words.
///
/// # Safety
/// The strings point at their stated lengths of readable bytes; `out` has
/// room for `cap` bytes and `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_authority_v1(
    act: Str,
    arguments: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    abi::guarded(status::PANIC, || unsafe {
        authority(act, arguments, out, cap, out_len)
    })
}

/// [`xmip_authority_v1`] within its guard.
unsafe fn authority(
    act: Str,
    arguments: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract on both strings.
    let read = [act, arguments].map(|text| unsafe { scope_text(text) });
    let [Some(act), Some(arguments)] = read else {
        return status::MALFORMED;
    };
    let answered = serde_json::from_str::<Value>(if arguments.is_empty() {
        "{}"
    } else {
        arguments
    })
    .map_err(|failure| format!("the arguments are not a JSON object: {failure}"))
    .and_then(|given| authority::act(act, &given));
    let (code, text) = match answered {
        Ok(answer) => (status::OK, answer.to_string()),
        Err(refused) => (status::INVALID, refused),
    };
    // SAFETY: `out` has room for `cap` bytes and `out_len` is writable.
    unsafe { refuse(&text, out, cap, out_len) };
    code
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use serde_json::json;

    use super::*;
    use crate::ffi::text::borrow;

    fn shown(path: &Path) -> String {
        path.display().to_string()
    }

    fn asked(act: &str, arguments: &Value) -> (i32, Value) {
        let text = arguments.to_string();
        let mut room = vec![0u8; 65_536];
        let mut length = 0usize;
        // SAFETY: both strings are borrowed for the call and the room is ours.
        let code = unsafe {
            xmip_authority_v1(
                borrow(act),
                borrow(&text),
                room.as_mut_ptr(),
                room.len(),
                &raw mut length,
            )
        };
        let said = String::from_utf8_lossy(&room[..length]).to_string();
        let value = serde_json::from_str(&said).unwrap_or(Value::String(said));
        (code, value)
    }

    #[test]
    fn an_authority_is_set_up_and_a_node_carried_across_an_air_gap_is_registered_and_revoked() {
        let base = file::temporary::Folder::new("operate-ffi-authority");
        let authority = base.join("authority");
        let node = base.join("node");
        let (code, set) = asked(
            "set-up",
            &json!({
                "authority": shown(&authority),
                "name": "test cluster authority",
                "publish": "http://ca.example:8080",
            }),
        );
        assert_eq!(code, status::OK, "{set}");
        assert_eq!(set["published"], "http://ca.example:8080");

        let (code, asked_for) = asked(
            "request",
            &json!({ "node": "r1", "names": ["r1.example"], "folder": shown(&node) }),
        );
        assert_eq!(code, status::OK, "{asked_for}");
        let out = base.join("r1.pem");
        let (code, signed) = asked(
            "sign",
            &json!({
                "authority": shown(&authority),
                "request": asked_for["request"],
                "names": ["r1", "r1.example"],
                "out": shown(&out),
            }),
        );
        assert_eq!(code, status::OK, "{signed}");
        let (code, imported) = asked(
            "import",
            &json!({ "certificate": shown(&out), "folder": shown(&node) }),
        );
        assert_eq!(code, status::OK, "{imported}");
        assert_eq!(imported["anchorTaken"], true);

        let (code, read) = asked("read", &json!({ "certificate": imported["certificate"] }));
        assert_eq!(
            (code, read["standing"].as_str()),
            (status::OK, Some("current"))
        );
        let (code, revoked) = asked(
            "revoke",
            &json!({
                "authority": shown(&authority),
                "serial": signed["serial"],
                "reason": "keyCompromise",
            }),
        );
        assert_eq!(code, status::OK, "{revoked}");
        let (_, inspected) = asked("inspect", &json!({ "authority": shown(&authority) }));
        assert_eq!(inspected["list"]["revoked"], 1);
        assert_eq!(inspected["issued"][0]["standing"], "revoked");

        let (code, refused) = asked("rotate", &json!({}));
        assert_eq!(code, status::INVALID);
        assert!(refused.as_str().is_some_and(|said| said.contains("no act")));
        let anchor = base.join("anchor.cer");
        let export =
            json!({ "authority": shown(&authority), "path": shown(&anchor), "format": "der" });
        assert_eq!(asked("export", &export).0, status::OK);
        assert_eq!(
            asked("export", &export).0,
            status::INVALID,
            "never over a file"
        );
    }
}
