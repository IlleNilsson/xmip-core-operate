//! `xmip_operate.h` section 28.2: a surface, `operate::surface`'s snapshot
//! and listener, chosen by `operate::surface::choice`, opened, read,
//! followed and asked to act, for every surface in another language — the
//! one the command line reads in Rust. Each export reads what a surface
//! handed over, calls the surface and writes the answer; none decides.
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16).
#![allow(unsafe_code)]

mod asked;
mod chosen;
mod opened;
mod read;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use abi::operate::surface::Surface as Handle;
use xcore::lock::Locked;

use crate::surface::SurfaceChange;
use crate::surface::choice::Chosen;
use crate::surface::publication_reading::PublicationReading;

/// What a follow has told: how many times, the revision and source it last
/// told.
#[derive(Default)]
struct Told {
    times: u64,
    revision: u64,
}

/// A surface handed across, and the one follow of it, started the first
/// time a caller waits and ended when the surface is freed.
pub(crate) struct Held {
    surface: Arc<Chosen>,
    told: Arc<(Mutex<Told>, Condvar)>,
    stop: Arc<AtomicBool>,
    following: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// The reading last handed across, and which it was.
    handed: Mutex<(Option<Arc<PublicationReading>>, u64)>,
}

impl Held {
    fn over(surface: Chosen) -> Self {
        Self {
            surface: Arc::new(surface),
            told: Arc::default(),
            stop: Arc::new(AtomicBool::new(false)),
            following: Mutex::new(None),
            handed: Mutex::new((None, 0)),
        }
    }

    // The surface's own follow (`OperatorSurface::watch`) on a thread of its
    // own, each telling counted: the one follow every waiter shares.
    fn follow(&self) {
        let mut following = self.following.locked();
        if following.is_some() {
            return;
        }

        let (surface, told, stop) = (
            Arc::clone(&self.surface),
            Arc::clone(&self.told),
            Arc::clone(&self.stop),
        );
        let started = std::thread::Builder::new()
            .name("xmip-surface-follow".to_string())
            .spawn(move || {
                surface
                    .surface()
                    .watch(&stop, &mut |change: &SurfaceChange| {
                        let (state, changed) = &*told;
                        let mut state = state.locked();
                        state.times += 1;
                        state.revision = change.revision;
                        changed.notify_all();
                    });
            });
        *following = started.ok();
    }

    // Once told more than `seen` times, or once `within` passed: how many
    // times and the revision last told.
    fn wait(&self, seen: u64, within: Duration) -> (u64, u64) {
        self.follow();
        let (state, changed) = &*self.told;
        let state =
            match changed.wait_timeout_while(state.locked(), within, |state| state.times <= seen) {
                Ok((state, _)) => state,
                Err(poisoned) => poisoned.into_inner().0,
            };

        (state.times, state.revision)
    }
}

impl Drop for Held {
    fn drop(&mut self) {
        // The follow is ended and waited for, a node's follow woken at once,
        // so nothing the surface lent — a keeper's callback — is called
        // after its release returns.
        self.stop.store(true, Ordering::Release);
        if let Chosen::Listener(node) = &*self.surface {
            node.end();
        }
        if let Some(follow) = self.following.locked().take() {
            let _ = follow.join();
        }
    }
}

/// `surface` handed across as a handle the caller frees.
pub(crate) fn hand(surface: Chosen) -> *mut Handle {
    Box::into_raw(Box::new(Held::over(surface))).cast::<Handle>()
}

/// The surface behind a handle.
///
/// # Safety
/// `surface` is null or a handle this section handed out and nobody freed.
pub(crate) unsafe fn held<'a>(surface: *const Handle) -> Option<&'a Held> {
    // SAFETY: per the contract above.
    unsafe { surface.cast::<Held>().as_ref() }
}

/// Release a surface and end its follow.
///
/// # Safety
/// `surface` is null or a handle this section handed out and nobody freed;
/// nothing borrowed from it is used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_free_v1(surface: *mut Handle) {
    if !surface.is_null() {
        // SAFETY: per the contract above, the box this handle was made from.
        let held = unsafe { Box::from_raw(surface.cast::<Held>()) };
        abi::guarded((), || drop(held));
    }
}

/// Wait until the surface's follow has told more than `seen` times, or
/// `within` milliseconds passed; how many times it told and the revision it
/// last told. The first wait starts the follow, which tells once at once.
///
/// # Safety
/// `surface` is a live handle; `out_told` and `out_revision` are writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_surface_wait_v1(
    surface: *const Handle,
    seen: u64,
    within: u32,
    out_told: *mut u64,
    out_revision: *mut u64,
) -> i32 {
    abi::guarded(abi::ffi::status::PANIC, || {
        // SAFETY: per the contract above.
        let Some(held) = (unsafe { held(surface) }) else {
            return abi::ffi::status::INVALID;
        };
        let (told, revision) = held.wait(seen, Duration::from_millis(u64::from(within)));
        // SAFETY: both are writable per the contract.
        unsafe {
            *out_told = told;
            *out_revision = revision;
        }
        abi::ffi::status::OK
    })
}

#[cfg(test)]
mod tests;
