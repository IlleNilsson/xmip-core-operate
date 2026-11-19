//! What a surface is asked to do, in section 28.2's one asking shape: an
//! act on one of a node's nouns or a failed Message, a scope paused or
//! resumed, the Messages that failed listed, a node's secrets listed or one
//! set — each as the caller a surface holds (0: this side), what is asked as
//! JSON in, what came of it as JSON out.

use abi::ffi::{Str, status};
use abi::operate::surface::Surface as Handle;
use abi::operate::words::outcome::{ASKED, REFUSED, TAKEN};
use observe::{Act, Noun};
use serde_json::{Value, json};

use super::{Held, held};
use crate::ffi::rule::refuse;
use crate::ffi::text::scope_text;
use crate::surface::acted::{ActOutcome, Acted, ScopeAction};
use crate::surface::choice::Chosen;
use crate::surface::failed_message_list::FailedMessageList;

/// Read the surface and both texts, answer, and write the answer or the
/// refusal: `XMIP_OK` with the answer, `XMIP_E_INVALID` with the refusal.
///
/// # Safety
/// `surface` is a live handle; `input` and `argument` point at their stated
/// lengths; `out` has room for `cap` bytes; `out_len` is writable.
unsafe fn asking(
    surface: *const Handle,
    (input, argument): (Str, Str),
    (out, cap, out_len): (*mut u8, usize, *mut usize),
    answering: impl FnOnce(&Held, &str, &str) -> Result<String, String>,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(held) = (unsafe { held(surface) }) else {
            return status::INVALID;
        };
        // SAFETY: per the contract above.
        let (Some(input), Some(argument)) = (unsafe { scope_text(input) }, unsafe {
            scope_text(argument)
        }) else {
            return status::MALFORMED;
        };
        let (code, text) = match answering(held, input, argument) {
            Ok(answer) => (status::OK, answer),
            Err(refusal) => (status::INVALID, refusal),
        };
        // SAFETY: per the contract above.
        unsafe { refuse(&text, out, cap, out_len) };
        code
    })
}

/// `act` on the `noun` called `target` of the node at `node`, the input
/// `{"node","noun","target","act"}` in the runtime's words, as `caller`:
/// `{"outcome","said"}`, the outcome `XMIP_ACTED_TAKEN`, `_ASKED` or
/// `_REFUSED` (section 28.3). A snapshot takes no act and says why.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_act_v1(
    surface: *const Handle,
    caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, input, _| {
                let asked: Value =
                    serde_json::from_str(input).map_err(|error| error.to_string())?;
                let text = |name: &str| asked[name].as_str().unwrap_or_default().to_string();
                let noun = Noun::named(&text("noun")).ok_or("no noun the runtime names")?;
                let act = Act::named(&text("act")).ok_or("no act the runtime names")?;
                let acted = match &*held.surface {
                    Chosen::Listener(node) => {
                        node.act_as(caller, &text("node"), noun, &text("target"), act)
                    }
                    Chosen::Snapshot(snapshot) => {
                        crate::surface::OperatorSurface::act(snapshot, "", noun, "", act, "")
                    }
                };
                Ok(said(&acted).to_string())
            },
        )
    }
}

/// `act` on the failed Message of `journey` at `scope` — the node that
/// sends its Send Port, or the Port's scope — the input
/// `{"scope","journey","act"}`, as `caller`: `{"node","outcome","said"}`,
/// the node it was asked of, refused before any node is asked where the
/// scope is on none or no Journey is named.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_act_message_v1(
    surface: *const Handle,
    caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, input, _| {
                let asked: Value =
                    serde_json::from_str(input).map_err(|error| error.to_string())?;
                let text = |name: &str| asked[name].as_str().unwrap_or_default().to_string();
                let act = Act::named(&text("act")).ok_or("no act the runtime names")?;
                let (scope, journey) = (text("scope"), text("journey"));
                let (node, acted) = match &*held.surface {
                    Chosen::Listener(node) => node.act_message_as(caller, &scope, &journey, act),
                    Chosen::Snapshot(snapshot) => crate::surface::OperatorSurface::act_message(
                        snapshot, &scope, &journey, act, "",
                    ),
                };
                let mut answer = said(&acted);
                answer["node"] = Value::String(node);
                Ok(answer.to_string())
            },
        )
    }
}

/// Pause or resume a scope, the input `{"scope","action","who"}`, the
/// action `pause` or `resume`: `{"applied","result"}`.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_control_v1(
    surface: *const Handle,
    _caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, input, _| {
                let asked: Value =
                    serde_json::from_str(input).map_err(|error| error.to_string())?;
                let text = |name: &str| asked[name].as_str().unwrap_or_default().to_string();
                let action = match text("action").as_str() {
                    "pause" => ScopeAction::Pause,
                    "resume" => ScopeAction::Resume,
                    _ => return Err("the action is pause or resume".to_string()),
                };
                let done = held
                    .surface
                    .surface()
                    .control(&text("scope"), action, &text("who"));
                Ok(json!({ "applied": done.applied, "result": done.result }).to_string())
            },
        )
    }
}

/// The Messages that failed at or beneath a scope, the input
/// `{"scope","from","most"}`: each Port's page from `from`, at most `most`
/// (0: every one carried), as `{"failed_messages":[...],"listed","failure"}`
/// in section 16's shape.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_failed_v1(
    surface: *const Handle,
    _caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, input, _| {
                let asked: Value =
                    serde_json::from_str(input).map_err(|error| error.to_string())?;
                let scope = asked["scope"].as_str().unwrap_or_default();
                let from = asked["from"].as_u64().unwrap_or(0);
                let most = u32::try_from(asked["most"].as_u64().unwrap_or(0)).unwrap_or(u32::MAX);
                let failed = held.surface.surface().failed_messages(scope, from, most);
                Ok(listed(&failed).to_string())
            },
        )
    }
}

/// The secrets the node at the input keeps, never a value, asked as
/// `caller`: `{"secrets":[{"name","kind","set_unix_nanos","set_by"}]}`; the
/// refusal where they were not listed.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_secrets_v1(
    surface: *const Handle,
    caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, node, _| {
                let listed = match &*held.surface {
                    Chosen::Listener(listener) => listener.secrets_as(caller, node)?,
                    Chosen::Snapshot(snapshot) => {
                        crate::surface::OperatorSurface::secrets(snapshot, node)?
                    }
                };
                let secrets: Vec<Value> = listed
                    .iter()
                    .map(|secret| {
                        json!({
                            "name": secret.name,
                            "kind": secret.kind,
                            "set_unix_nanos": secret.set_unix_nanos,
                            "set_by": secret.set_by,
                        })
                    })
                    .collect();
                Ok(json!({ "secrets": secrets }).to_string())
            },
        )
    }
}

/// Set a secret through a node, the input `{"node","name","kind"}`, the
/// value the argument — never written anywhere but to the node — asked as
/// `caller`: `{"applied","result"}`.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_set_secret_v1(
    surface: *const Handle,
    caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, input, value| {
                let asked: Value =
                    serde_json::from_str(input).map_err(|error| error.to_string())?;
                let text = |name: &str| asked[name].as_str().unwrap_or_default().to_string();
                let given = (text("kind"), value);
                let set = match &*held.surface {
                    Chosen::Listener(node) => node.set_secret_as(
                        caller,
                        &text("node"),
                        &text("name"),
                        (&given.0, given.1),
                    ),
                    Chosen::Snapshot(snapshot) => crate::surface::OperatorSurface::set_secret(
                        snapshot,
                        &text("node"),
                        &text("name"),
                        (&given.0, given.1),
                        "",
                    ),
                };
                Ok(json!({ "applied": set.applied, "result": set.result }).to_string())
            },
        )
    }
}

// What came of an act, as JSON.
fn said(acted: &Acted) -> Value {
    let outcome = match acted.outcome {
        ActOutcome::Taken => TAKEN,
        ActOutcome::Asked => ASKED,
        ActOutcome::Refused => REFUSED,
    };
    json!({ "outcome": outcome, "said": acted.said })
}

/// A list of the Messages that failed as section 16's JSON, with whether it
/// is one and why it was not answered.
pub(crate) fn listed(failed: &FailedMessageList) -> Value {
    let ports: Vec<Value> = failed
        .ports
        .iter()
        .map(|port| {
            json!({
                "node": port.node,
                "send_port": port.send_port,
                "count": port.count,
                "next": port.next,
                "blocked": port.blocked,
                "messages": port.messages.iter().map(|message| json!({
                    "journey": message.journey,
                    "sequence": message.sequence,
                    "reason": message.reason,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();
    json!({ "failed_messages": ports, "listed": failed.listed, "failure": failed.failure })
}

/// How the surface stands, nothing asked of a node:
/// `{"connected","reason","gathers"}` — whether its node answered when last
/// asked (a snapshot: whether its file is there), why not where it did not,
/// and whether it follows the cluster's view its node gathers.
///
/// # Safety
/// As `asking`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_standing_v1(
    surface: *const Handle,
    _caller: u64,
    input: Str,
    argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is this export's, passed on whole.
    unsafe {
        asking(
            surface,
            (input, argument),
            (out, cap, out_len),
            |held, _, _| {
                let (connected, reason, gathers) = match &*held.surface {
                    Chosen::Listener(node) => node.standing(),
                    Chosen::Snapshot(snapshot) => (snapshot.exists(), String::new(), false),
                };
                Ok(
                    json!({ "connected": connected, "reason": reason, "gathers": gathers })
                        .to_string(),
                )
            },
        )
    }
}
