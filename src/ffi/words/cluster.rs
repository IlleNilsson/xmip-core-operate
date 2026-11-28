//! A cluster as every surface shows it, from its JSON in section 10's
//! shape: `operate::surface::cluster_named`'s rule, for the surfaces in
//! another language (ADR-0028, amendment 2026-11-28).
#![allow(unsafe_code)]

use abi::ffi::Str;
use observe::Naming;
use serde_json::{Value, json};

use crate::ffi::text::answer;
use crate::surface::cluster_named::ClusterNamed;

/// `xmip_cluster_said_v1`: the cluster `input` names — `{"name",
/// "short_name", "file"}`, each a string, any left out empty — as
/// `{"shown", "title"}`: what a surface shows for it, and the tooltip
/// beside it.
///
/// # Safety
/// As section 10's shape: both texts readable for their lengths; `out` has
/// room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_cluster_said_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, _| {
            cluster_said(input)
        })
    }
}

fn cluster_said(input: &str) -> Result<String, String> {
    let given: Value =
        serde_json::from_str(input).map_err(|failure| format!("not a cluster: {failure}"))?;
    let text = |key: &str| given[key].as_str().unwrap_or_default().to_string();
    let named = ClusterNamed {
        name: text("name"),
        naming: Naming {
            short_name: text("short_name"),
            file: text("file"),
        },
    };
    Ok(json!({ "shown": named.shown(), "title": named.title() }).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::operate::words::SaidFn;

    #[test]
    fn the_export_has_the_shape_the_binding_declares_and_says_the_rule() {
        let _: SaidFn = xmip_cluster_said_v1;
        let cluster = configure::fixture::test_cluster();
        let input = json!({ "name": cluster.name, "short_name": "Ab", "file": "c/a.xmip" });
        let said: Value =
            serde_json::from_str(&cluster_said(&input.to_string()).expect("said")).expect("JSON");
        assert_eq!(said["shown"], "Ab");
        assert_eq!(
            said["title"],
            format!(
                "a · short name Ab · cluster {} · from c/a.xmip",
                cluster.name
            )
        );
        let bare: Value = serde_json::from_str(
            &cluster_said(&json!({ "name": cluster.name }).to_string()).expect("said"),
        )
        .expect("JSON");
        assert_eq!(bare["shown"], cluster.name);
        assert!(cluster_said("not json").is_err());
    }
}
