//! What every operator surface calls, implemented once (ADR-0058, amendment
//! 2026-11-16): the C ABI's operating exports over the runtime,
//! `xmip_operate.h` ([`ffi`]), in the native library the command line, the
//! PowerShell module, the GUIs and the designer's language server load; the
//! operator a surface's table stands for and what this process has published
//! ([`operator`]); the header's integers for the crates' enums (`wire`); and
//! the designer's view of `cluster.xmip`, its edits and what an editor
//! suggests ([`design`]).
//!
//! Two builds (ADR-0014, amendment 2026-10-14), as the runtime's. The `node`
//! feature, the default, is every export. The `remote` feature alone is a
//! surface that follows nodes over their operate listener and runs none, the
//! phone: the exports a surface reads with (`xmip_operate.h` sections 7, 8,
//! 9, 18 and section 17's client).

#[cfg(feature = "remote")]
pub mod audit_read;
#[cfg(feature = "node")]
pub mod authority;
#[cfg(feature = "node")]
pub mod certificate;
#[cfg(feature = "node")]
pub mod database;
#[cfg(feature = "node")]
pub mod design;
#[cfg(feature = "remote")]
pub mod document;
pub mod ffi;
#[cfg(feature = "node")]
pub mod module_probe;
#[cfg(feature = "node")]
pub mod operator;
#[cfg(feature = "remote")]
pub mod surface;
#[cfg(feature = "remote")]
mod wire;
