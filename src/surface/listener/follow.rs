//! What a listener follows, told: the runtime's follow holds the node's
//! wait, and each time it tells, what it follows is taken from it, the
//! reading held goes and the watch is told at once — read by whoever asks
//! next, never before the telling. A lost node is told too, and the follow
//! tries it again on its own loop (ADR-0052, note of 2026-10-05), deciding
//! afresh what it follows as it reconnects.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use runtime::plane::remote::Follows;
use xcore::lock::Locked;

use super::ListenerSurface;
use crate::surface::{OperatorSurface, SurfaceChange};

/// How long a follow waits for the node to tell before it looks again.
const TURN: Duration = Duration::from_secs(1);

/// How long a surface with no node waits before it looks whether its
/// watch ended.
const IDLE: Duration = Duration::from_millis(100);

impl ListenerSurface {
    /// Tell `each` what the surface is at now, then once each time the
    /// runtime's follow tells, until `stop`.
    pub(super) fn follow(&self, stop: &AtomicBool, each: &mut dyn FnMut(&SurfaceChange)) {
        each(&SurfaceChange {
            revision: 0,
            source: self.source(),
        });
        let Some(node) = &self.node else {
            while !stop.load(Ordering::Acquire) {
                std::thread::sleep(IDLE);
            }
            return;
        };

        self.following.store(true, Ordering::Release);
        let mut seen = 0;

        while !stop.load(Ordering::Acquire) {
            let Ok(told) = node.follow(Follows::Answered, seen, TURN) else {
                continue;
            };
            if told.told == seen {
                continue;
            }
            seen = told.told;
            {
                let mut state = self.state.locked();
                state.follows = told.follows;
                state.told_revision = told.revision;
                state.held = None;
                if told.follows != Follows::Cluster {
                    state.gathered = String::new();
                }
                state.connected = told.reachable;
                state.reason = if told.reachable {
                    String::new()
                } else {
                    told.reason
                };
            }
            each(&SurfaceChange {
                revision: told.revision,
                source: self.source(),
            });
        }

        self.following.store(false, Ordering::Release);
    }

    /// End what follows the node, waking every wait on it at once.
    pub fn end(&self) {
        if let Some(node) = &self.node {
            node.end();
        }
    }
}

impl Drop for ListenerSurface {
    fn drop(&mut self) {
        self.end();
    }
}
