//! A run's line and what it started each node with, and a node's
//! declaration as a surface says it, each from its JSON in section 10's
//! shape: `operate::surface::run_header`'s and `node_capability`'s.
#![allow(unsafe_code)]

use abi::ffi::Str;
use observe::Run;
use serde_json::{Value, json};

use crate::ffi::text::answer;
use crate::surface::node_capability::NodeCapability;
use crate::surface::run_header::RunHeader;

/// `xmip_run_said_v1`: the run `input` says — `observe::Run`'s JSON,
/// `cluster`, `tests`, `nodes`, `roles`, `capabilities`, `real`,
/// `stress` — as `{"said", "line", "started": [{"node", "roles",
/// "capabilities", "refusal"}]}`, one started node for each of its roles'
/// entries.
///
/// # Safety
/// As section 10's shape: both texts readable for their lengths; `out` has
/// room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_run_said_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, _| {
            run_said(input)
        })
    }
}

fn run_said(input: &str) -> Result<String, String> {
    let run: Run =
        serde_json::from_str(input).map_err(|failure| format!("not a run: {failure}"))?;
    let header = RunHeader::from(Some(&run));
    let started: Vec<Value> = run
        .roles
        .iter()
        .map(|entry| {
            let node = NodeCapability::started(entry).node;
            let capability = header.capability(&node);
            json!({
                "node": capability.node,
                "roles": capability.roles,
                "capabilities": capability.capabilities,
                "refusal": capability.refusal,
            })
        })
        .collect();

    Ok(json!({ "said": header.said(), "line": header.line(), "started": started }).to_string())
}

/// `xmip_capability_said_v1`: a node's declaration `input` —
/// `{"node", "roles", "capabilities", "published", "refusal"}` — as a
/// surface says it: `{"said", "words", "line", "purpose", "stages",
/// "named", "origin"}`, `named` its Node capabilities in one phrase.
///
/// # Safety
/// As section 10's shape: both texts readable for their lengths; `out` has
/// room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_capability_said_v1(
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(input, argument, out, cap, out_len, |input, _| {
            capability_said(input)
        })
    }
}

fn capability_said(input: &str) -> Result<String, String> {
    let given: Value =
        serde_json::from_str(input).map_err(|failure| format!("not a declaration: {failure}"))?;
    let text = |key: &str| given[key].as_str().unwrap_or_default().to_string();
    let words = |key: &str| -> Vec<String> {
        given[key]
            .as_array()
            .map(|words| {
                words
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    };
    let capability = NodeCapability {
        node: text("node"),
        roles: words("roles"),
        capabilities: words("capabilities"),
        published: given["published"].as_bool().unwrap_or(false),
        evidence: text("evidence"),
        refusal: text("refusal"),
    };

    Ok(json!({
        "said": capability.said(),
        "words": capability.words(),
        "line": capability.line(),
        "purpose": capability.purpose(),
        "stages": capability.stages(),
        "named": capability.named(),
        "origin": capability.origin(),
    })
    .to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::operate::words::SaidFn;

    #[test]
    fn the_exports_have_the_shape_the_binding_declares() {
        let _: SaidFn = xmip_run_said_v1;
        let _: SaidFn = xmip_capability_said_v1;
    }

    #[test]
    fn a_run_is_said_with_each_node_it_started_and_a_declaration_in_its_words() {
        let test = configure::fixture::test_cluster();
        let a = &test.with_role("receiving").name;
        let b = &test
            .nodes
            .iter()
            .find(|node| &node.name != a)
            .expect("a second node")
            .name;
        let run = json!({
            "cluster": test.name, "tests": ["t"], "nodes": [a, b],
            "roles": [format!("{a}=receiving"), format!("{b}=fly")],
            "capabilities": [format!("{a}=online")], "stress": "light",
        })
        .to_string();
        let said: Value = serde_json::from_str(&run_said(&run).expect("said")).expect("JSON");

        assert_eq!(
            said["line"],
            format!(
                "t · {} · nodes {a}=receiving {b}=fly · Node capabilities {a}=online · light",
                test.name
            )
        );
        assert_eq!(said["started"][0]["capabilities"], json!(["online"]));
        assert_eq!(said["started"][0]["roles"], json!(["receiving"]));
        assert!(
            said["started"][1]["refusal"]
                .as_str()
                .is_some_and(|it| !it.is_empty())
        );

        let declared = json!({
            "node": a, "roles": ["receiving"], "capabilities": ["online", "erp-direct"],
            "published": true
        })
        .to_string();
        let said: Value =
            serde_json::from_str(&capability_said(&declared).expect("said")).expect("JSON");
        assert_eq!(said["line"], "receiving · online, erp-direct");
        assert_eq!(said["named"], "online, erp-direct");
        assert_eq!(said["origin"], "published by the node");
        assert_eq!(said["stages"], json!(["receive"]));
        assert!(run_said("not json").is_err());
    }
}
