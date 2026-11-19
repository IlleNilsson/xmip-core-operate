//! Section 28.2's surfaces opened — over a published snapshot, a node at its
//! local endpoint, a node at its operate address presenting files or a key
//! its keeper holds — and asked where they read from and whether they
//! answer. Telling time by the operating system's clock, as every surface a
//! program opens across the boundary does (`ffi::remote::hosted_clock`).

use std::path::PathBuf;
use std::time::Duration;

use abi::ffi::{Str, status};
use abi::operate::remote::SignFn;
use abi::operate::surface::Surface as Handle;

use super::{hand, held};
use crate::ffi::keeper::kept;
use crate::ffi::remote::hosted_clock;
use crate::ffi::rule::refuse;
use crate::ffi::text::scope_text;
use crate::surface::choice::Chosen;
use crate::surface::choice::named::Named;
use crate::surface::listener::ListenerSurface;
use crate::surface::snapshot::SnapshotSurface;

/// The surface over the publication at `path`, read as it changes.
///
/// # Safety
/// `path` points at its stated length of readable bytes; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_snapshot_v1(
    path: Str,
    _within: u32,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(path) = (unsafe { scope_text(path) }) else {
            return status::MALFORMED;
        };
        let surface = Chosen::Snapshot(SnapshotSurface::new(PathBuf::from(path), hosted_clock()));
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(surface) };
        status::OK
    })
}

/// The surface over the node on this machine at its local endpoint
/// `endpoint` — none where it is empty, which every answer says — each call
/// waiting at most `within` milliseconds, 0 for what the node declared.
///
/// # Safety
/// `endpoint` points at its stated length of readable bytes; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_local_v1(
    endpoint: Str,
    within: u32,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(endpoint) = (unsafe { scope_text(endpoint) }) else {
            return status::MALFORMED;
        };
        let within = Duration::from_millis(u64::from(within));
        let node = ListenerSurface::local_within(Some(endpoint), within, hosted_clock());
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(Chosen::Listener(node)) };
        status::OK
    })
}

/// The surface over the node at `address`, presenting the PEM files the
/// paths name — empty where none is named — and trusting the anchors there.
/// Files that do not make an identity are said on every answer, as a node
/// that does not answer is.
///
/// # Safety
/// The strings point at their stated lengths of readable bytes; `out` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_remote_v1(
    address: Str,
    certificate: Str,
    key: Str,
    anchors: Str,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some([address, certificate, key, anchors]) =
            (unsafe { crate::ffi::remote::texts([address, certificate, key, anchors]) })
        else {
            return status::MALFORMED;
        };
        let named = Named {
            certificate: path(certificate),
            key: path(key),
            anchors: path(anchors),
            kept: false,
        };
        let node = ListenerSurface::remote(address, named.identity(), hosted_clock());
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(Chosen::Listener(node)) };
        status::OK
    })
}

/// The surface over the node at `address`, presenting the chain at the path
/// `certificate`, whose key the keeper `keeper` names holds: every
/// signature asked of `sign` with `context`; trusting the anchors at
/// `anchors`.
///
/// # Safety
/// The strings point at their stated lengths of readable bytes; `sign` may
/// be called with `context` from any thread, at once from several, until
/// the surface is freed; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_remote_kept_v1(
    address: Str,
    certificate: Str,
    anchors: Str,
    keeper: Str,
    sign: SignFn,
    context: *mut core::ffi::c_void,
    out: *mut *mut Handle,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some([address, certificate, anchors, keeper]) =
            (unsafe { crate::ffi::remote::texts([address, certificate, anchors, keeper]) })
        else {
            return status::MALFORMED;
        };
        let named = Named {
            certificate: path(certificate),
            key: None,
            anchors: path(anchors),
            kept: true,
        };
        // SAFETY: per the contract above, `sign` may be called with
        // `context` until the surface is freed, which outlives the signer.
        let signer = unsafe { kept(sign, context) };
        let identity = named.kept_identity(keeper, signer);
        let node = ListenerSurface::remote(address, identity, hosted_clock());
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = hand(Chosen::Listener(node)) };
        status::OK
    })
}

/// Where the surface's answers come from, in words, as UTF-8 in `out`, its
/// true length in `out_len`.
///
/// # Safety
/// `surface` is a live handle; `out` has room for `cap` bytes; `out_len` is
/// writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_source_v1(
    surface: *const Handle,
    _caller: u64,
    _input: Str,
    _argument: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(held) = (unsafe { held(surface) }) else {
            return status::INVALID;
        };
        // SAFETY: per the contract above.
        unsafe { refuse(&held.surface.source(), out, cap, out_len) };
        status::OK
    })
}

/// Whether the surface answers now — a node that answers, a snapshot whose
/// file is there — 1 or 0 in `out`.
///
/// # Safety
/// `surface` is a live handle; `out` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_answers_v1(surface: *const Handle, out: *mut u8) -> i32 {
    abi::guarded(status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(held) = (unsafe { held(surface) }) else {
            return status::INVALID;
        };
        // SAFETY: `out` is writable per the contract.
        unsafe { *out = u8::from(held.surface.answers()) };
        status::OK
    })
}

// A path handed across, `None` where it is empty.
fn path(text: &str) -> Option<PathBuf> {
    (!text.trim().is_empty()).then(|| PathBuf::from(text))
}
