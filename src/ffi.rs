//! The operating C boundary, in one folder (ADR-0050, amendment
//! 2026-11-16): the operator's table and the exports a surface calls
//! (`xmip_operate.h` sections 1 and 5 to 27, ADR-0027, ADR-0053, ADR-0064,
//! ADR-0065, ADR-0013, ADR-0034, ADR-0072, ADR-0012 and ADR-0031), each
//! over the runtime. The
//! local operate endpoint's pipe on Windows is `net::pipe`'s, the estate's
//! one (ADR-0014, amendment 2026-10-21). Unsafe code is allowed in these
//! files and nowhere else in this crate; each file lowers the crate's `deny`
//! at its top, and `test/Unsafe.Test.ps1` holds the folder.
//!
//! A remote build (`remote` without `node`) carries what a surface reads
//! and records with: sections 7, 8, 9, 18 and 27, section 17's client,
//! section 19's request, and of sections 11, 14, 15 and 16 only what a read
//! publication carries; the rest is the node's.

#[cfg(feature = "remote")]
pub mod audit;
#[cfg(feature = "node")]
pub mod authority;
#[cfg(feature = "node")]
pub mod catalogue;
#[cfg(feature = "remote")]
pub mod certificate;
#[cfg(feature = "remote")]
pub mod curve;
#[cfg(feature = "node")]
pub mod database;
#[cfg(feature = "remote")]
pub mod dead_message;
#[cfg(feature = "node")]
pub mod design;
#[cfg(feature = "remote")]
pub mod document;
#[cfg(feature = "remote")]
pub mod event;
#[cfg(feature = "remote")]
pub mod from_to;
#[cfg(feature = "remote")]
pub mod instance;
#[cfg(feature = "remote")]
pub mod keeper;
#[cfg(feature = "remote")]
pub mod layout;
#[cfg(feature = "remote")]
pub mod message;
#[cfg(feature = "node")]
pub mod module_probe;
#[cfg(feature = "node")]
pub mod node_certificate;
#[cfg(feature = "node")]
pub mod process;
#[cfg(feature = "remote")]
pub mod publication;
#[cfg(feature = "remote")]
pub mod remote;
#[cfg(feature = "node")]
pub mod report;
/// The library pins itself when Windows loads it; `build.rs` does the same
/// for `dlclose` (`xmip_operate.h` section 1).
#[cfg(windows)]
pub mod resident;
#[cfg(feature = "remote")]
pub mod rule;
#[cfg(feature = "remote")]
pub mod scope_index;
#[cfg(feature = "node")]
pub mod secure_listener;
#[cfg(feature = "remote")]
pub mod security_role;
#[cfg(feature = "node")]
pub mod served;
#[cfg(feature = "node")]
pub mod start;
#[cfg(feature = "remote")]
pub mod subscription;
#[cfg(feature = "remote")]
pub mod surface;
/// The operator's table, `xmip_operate_v1`.
#[cfg(feature = "node")]
pub mod table;
#[cfg(feature = "remote")]
mod text;
#[cfg(feature = "remote")]
pub mod topology;
#[cfg(feature = "remote")]
pub mod words;
#[cfg(feature = "node")]
pub mod written;
