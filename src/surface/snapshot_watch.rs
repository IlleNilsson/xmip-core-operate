//! A snapshot followed as its publisher replaces it: told by the operating
//! system each time the file's directory changes (`runtime::directory_watch`),
//! never by looking again and again (ADR-0052, amendment 2026-09-15). A
//! publication whose directory is not made yet is said so at once, and
//! followed from the moment it is.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, sync_channel};
use std::time::Duration;

use crate::surface::snapshot::SnapshotSurface;
use crate::surface::{OperatorSurface, SurfaceChange};

/// How often a wait looks whether it was asked to stop.
const TURN: Duration = Duration::from_millis(100);

/// How long a notice waits for the others one replacement makes.
const SETTLE: Duration = Duration::from_millis(25);

/// Follow `surface` until `stop` is set, telling `each` at once and then
/// each time its publication changes.
pub(crate) fn watch(
    surface: &SnapshotSurface,
    stop: &AtomicBool,
    each: &mut dyn FnMut(&SurfaceChange),
) {
    let full = std::path::absolute(surface.path()).unwrap_or_else(|_| surface.path().into());
    let directory = full
        .parent()
        .map_or_else(|| full.clone(), Path::to_path_buf);
    let waited = !directory.is_dir();
    let mut announced = 0;

    if waited {
        each(&told(0, surface));
        if !arrived(&directory, stop) {
            return;
        }
    }

    let (notice, notices) = sync_channel::<()>(1);
    let watch = watched(&directory, &notice);
    let polling = watch.is_none();

    if !surface.reread() {
        let _ = notice.try_send(());
    }

    if waited {
        announced += 1;
    }
    // What the first telling tells, held so the next is told only where
    // another was read: held, never an address an index freed may share.
    let mut last = surface.index();
    each(&told(announced, surface));
    let _watch = watch;

    while let Some(()) = next(&notices, stop, polling) {
        std::thread::sleep(SETTLE);
        while notices.try_recv().is_ok() {}

        if !surface.reread() {
            let _ = notice.try_send(());
            continue;
        }

        let now = surface.index();
        if Arc::ptr_eq(&now, &last) {
            continue;
        }
        last = now;
        announced += 1;
        each(&told(announced, surface));
    }
}

fn told(revision: u64, surface: &SnapshotSurface) -> SurfaceChange {
    SurfaceChange {
        revision,
        source: surface.source(),
    }
}

// The next notice — each turn where nothing tells — or `None` once asked
// to stop.
fn next(notices: &Receiver<()>, stop: &AtomicBool, polling: bool) -> Option<()> {
    loop {
        if stop.load(Ordering::Acquire) {
            return None;
        }
        match notices.recv_timeout(TURN) {
            Ok(()) => return Some(()),
            Err(RecvTimeoutError::Timeout) if polling => return Some(()),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                std::thread::sleep(TURN);
                return Some(());
            }
        }
    }
}

// The operating system's watch of `directory`, telling `notice`; where it
// will not watch, `None`, and the follow looks each turn.
#[cfg(feature = "node")]
fn watched(
    directory: &Path,
    notice: &SyncSender<()>,
) -> Option<runtime::directory_watch::DirectoryWatch> {
    let notice = notice.clone();
    runtime::directory_watch::DirectoryWatch::spawn(directory, move || {
        let _ = notice.try_send(());
    })
    .ok()
}

#[cfg(not(feature = "node"))]
fn watched(_directory: &Path, _notice: &SyncSender<()>) -> Option<()> {
    None
}

// Whether `directory` came to be before `stop` was set, waiting on the
// nearest folder above it that is.
fn arrived(directory: &Path, stop: &AtomicBool) -> bool {
    while !directory.is_dir() {
        if stop.load(Ordering::Acquire) {
            return false;
        }
        let Some(above) = nearest(directory) else {
            std::thread::sleep(TURN);
            continue;
        };
        let (notice, notices) = sync_channel::<()>(1);
        // Held for this turn of the wait, and ended with it: the watch tells
        // of a folder made as of a file (`runtime::directory_watch`); where
        // it cannot watch, each turn looks.
        let watch = watched(above, &notice);
        // A folder made while the watch attached was told to nobody: looked
        // for once the watch is there, and waited for only where none was.
        if directory.is_dir() || nearest(directory) != Some(above) {
            continue;
        }
        let _ = next(&notices, stop, watch.is_none());
    }

    true
}

// The nearest folder above `directory` that is there.
fn nearest(directory: &Path) -> Option<&Path> {
    directory.ancestors().skip(1).find(|above| above.is_dir())
}
