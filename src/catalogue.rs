//! What this library's catalogue carries: every technology's declaration
//! xmip-core-declaration gathers, carried into the runtime's catalogue once
//! in the process, before an export reads it (ADR-0064, amendments
//! 2026-10-22 and 2026-11-26).
//!
//! A surface's process loads no technology, so until 2026-11-26 the
//! catalogue it asked held none, and the language server and the desktop
//! suggested no Location's settings. The runtime cannot gather the
//! declarations itself, since a platform service depends on no technology
//! (`architecture.toml`); this library, the one the surfaces load, links the
//! gathering crate and hands each declaration to the runtime's one way in,
//! `runtime::catalogue::carry`, as a node hands each technology it loads.
//! From then on `xmip_technology_catalogue_v1` answers them, and validating
//! or planning a node in the process holds each Location to them.

use std::sync::Once;

use runtime::catalogue::{Capability, carry};

/// Carry every technology's declaration into the runtime's catalogue, the
/// first time a process asks; every later call does nothing.
pub fn carried() {
    static CARRIED: Once = Once::new();
    CARRIED.call_once(|| {
        for declared in declaration::TRANSPORTS {
            carry(Capability::Transport, declared.settings, declared.schemes);
        }
    });
}
