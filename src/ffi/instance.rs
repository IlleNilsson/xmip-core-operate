//! `xmip_operate.h` section 29, a Work Process's Instances: listed from the
//! Ledger or as a read publication carries them, forwarded (ADR-0066,
//! amendment 2026-11-25). A Retry or a Dismiss of a Message that failed at
//! a Work Process reaches a node through its operate listener's door and
//! Xmip Storage, as section 16's acts do, never here.
//!
//! The list is `runtime::work_step`'s. This writes one list's JSON, once,
//! for this process's nodes and for a read publication alike.
#![allow(unsafe_code)]

#[cfg(feature = "node")]
use abi::ffi::Str;
use abi::ffi::status;
use abi::operate::publication::Publication as Handle;
#[cfg(feature = "node")]
use node::Scope;
use observe::{WorkProcessInstance, WorkProcessInstances};
use serde_json::{Value, json};

use crate::ffi::publication::held;
use crate::ffi::rule::refuse;
#[cfg(feature = "node")]
use crate::ffi::text::scope_text;
#[cfg(feature = "node")]
use runtime::work_step::WorkStep;

/// One Instance as the header writes it.
fn instance(one: &WorkProcessInstance) -> Value {
    let history: Vec<Value> = one
        .history
        .iter()
        .map(|entry| {
            json!({
                "at_unix_nanos": entry.at_unix_nanos,
                "what": entry.what,
                "message": entry.message,
                "journey": entry.journey,
            })
        })
        .collect();
    json!({
        "instance": one.instance,
        "state": one.state,
        "waits_for": one.waits_for,
        "until_unix_nanos": one.until_unix_nanos,
        "correlations": one.correlations,
        "code": one.code,
        "reason": one.reason,
        "failed_by": one.failed_by,
        "values": one.values,
        "history": history,
        "earlier": one.earlier,
        "turns": one.turns,
        "updated_unix_nanos": one.updated_unix_nanos,
    })
}

/// A list of each Work Process's Instances as the header writes it.
pub(crate) fn listed<'a>(lists: impl Iterator<Item = &'a WorkProcessInstances>) -> String {
    let lists: Vec<Value> = lists
        .map(|list| {
            let failed: Vec<Value> = list
                .failed
                .iter()
                .map(|failed| {
                    json!({
                        "journey": failed.journey,
                        "sequence": failed.sequence,
                        "reason": failed.reason,
                    })
                })
                .collect();
            json!({
                "node": list.node,
                "work_process": list.work_process,
                "failed_count": list.failed_count,
                "failed": failed,
                "instances": list.instances.iter().map(instance).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({ "work_process_instances": lists }).to_string()
}

/// The Instances of `work_process` (empty: every one) of every node in this
/// process at or beneath `node` (empty: all), read from the Ledger, at most
/// `most` of each (0: `work_step::INSTANCE_PAGE`, a hundred), as
/// `runtime::work_step::WorkStep::instances` reads them.
///
/// # Safety
/// `node` and `work_process` point at their stated length of readable
/// bytes; `out` has room for `cap` bytes; `out_len` is writable.
#[unsafe(no_mangle)]
#[cfg(feature = "node")]
pub unsafe extern "C" fn xmip_instances_v1(
    node: Str,
    work_process: Str,
    most: u32,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    abi::guarded(status::PANIC, || unsafe {
        instances_listed(node, work_process, most, out, cap, out_len)
    })
}

/// [`xmip_instances_v1`] within its guard.
#[cfg(feature = "node")]
unsafe fn instances_listed(
    node: Str,
    work_process: Str,
    most: u32,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller upholds the header's contract on both strings.
    let texts = unsafe { (scope_text(node), scope_text(work_process)) };
    let (Some(node), Some(work_process)) = texts else {
        return status::MALFORMED;
    };
    let (code, text) = match instances(node, work_process, most) {
        Ok(lists) => (status::OK, listed(lists.iter())),
        Err(failed) => (status::IO, failed),
    };

    // SAFETY: `out` and `out_len` per the contract.
    unsafe { refuse(&text, out, cap, out_len) };
    code
}

/// Each Work Process's Instances, as [`xmip_instances_v1`] lists them.
#[cfg(feature = "node")]
fn instances(
    node: &str,
    work_process: &str,
    most: u32,
) -> Result<Vec<WorkProcessInstances>, String> {
    let within = Scope::new(node);
    let mut lists = Vec::new();
    for step in WorkStep::registered() {
        if !within.contains(Scope::new(step.node())) {
            continue;
        }
        let names: Vec<String> = step
            .running()
            .iter()
            .map(|running| running.name.clone())
            .filter(|name| work_process.is_empty() || name == work_process)
            .collect();
        for name in names {
            lists.push(step.instances(&name, most)?);
        }
    }
    Ok(lists)
}

/// What a read publication carries of Work Processes' Instances.
///
/// # Safety
/// `publication` is a live handle; `out` has room for `cap` bytes;
/// `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_publication_instances_v1(
    publication: *const Handle,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    abi::guarded(status::PANIC, || unsafe {
        publication_instances(publication, out, cap, out_len)
    })
}

/// [`xmip_publication_instances_v1`] within its guard.
unsafe fn publication_instances(
    publication: *const Handle,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: per the contract above.
    let Some(read) = (unsafe { held(publication) }) else {
        return status::INVALID;
    };
    let text = listed(read.publication.work_process_instances.iter());

    // SAFETY: `out` and `out_len` per the contract.
    unsafe { refuse(&text, out, cap, out_len) };
    status::OK
}

#[cfg(all(test, feature = "node"))]
mod tests {
    use super::*;
    use crate::ffi::text::borrow;

    #[test]
    fn the_exports_have_the_shapes_the_binding_declares() {
        let _: abi::operate::instance::InstancesFn = xmip_instances_v1;
        let _: abi::operate::instance::PublicationInstancesFn = xmip_publication_instances_v1;
    }

    #[test]
    fn no_node_here_lists_no_instances_and_a_list_has_the_headers_shape() {
        let node = format!(
            "{}-ffi-instances",
            configure::fixture::test_cluster().node_scope(0)
        );
        let mut buffer = vec![0u8; 1024];
        let mut length = 0usize;
        // SAFETY: every pointer is this test's own string or buffer, alive for the call.
        let code = unsafe {
            xmip_instances_v1(
                borrow(&node),
                borrow(""),
                0,
                buffer.as_mut_ptr(),
                buffer.len(),
                &raw mut length,
            )
        };
        assert_eq!(code, status::OK);
        let said: Value = serde_json::from_slice(&buffer[..length]).expect("JSON");
        assert_eq!(said["work_process_instances"], json!([]));

        let list = WorkProcessInstances {
            node,
            work_process: "Order in parts".to_string(),
            instances: vec![WorkProcessInstance {
                instance: "i".to_string(),
                state: "waiting".to_string(),
                waits_for: "message or time".to_string(),
                until_unix_nanos: Some(9),
                correlations: vec![["Order".to_string(), "A-1001".to_string()]],
                values: vec![["Delivered".to_string(), "2".to_string()]],
                turns: 3,
                ..WorkProcessInstance::default()
            }],
            failed_count: 1,
            failed: vec![observe::FailedMessage {
                journey: "j".to_string(),
                sequence: 4,
                reason: "threw offline".to_string(),
            }],
        };
        let read: Value = serde_json::from_str(&listed(std::iter::once(&list))).expect("JSON");
        let first = &read["work_process_instances"][0];
        assert_eq!(first["work_process"], "Order in parts");
        assert_eq!(first["failed_count"], 1);
        assert_eq!(first["failed"][0]["journey"], "j");
        let one = &first["instances"][0];
        assert_eq!(one["state"], "waiting");
        assert_eq!(one["until_unix_nanos"], 9);
        assert_eq!(one["correlations"][0], json!(["Order", "A-1001"]));
        assert_eq!(one["values"][0], json!(["Delivered", "2"]));
        assert_eq!(
            read.get("orders"),
            None,
            "a snapshot is read, never acted on"
        );
    }
}
