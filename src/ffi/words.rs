//! `xmip_operate.h` section 28.3: what every surface says and how it orders
//! a list, `operate::surface`'s english, run_header, node_capability,
//! configuration_verdict, acted, the noun queries and list_order, and abi's
//! status_meaning and boundaries (ADR-0052, amendment 2026-11-18). Each
//! export reads what a surface handed over, calls the rule, and writes the
//! answer where it was told; none decides. The fixed words are static, lent
//! for the life of the library (`said`); a sentence is written into the
//! caller's buffer (`sentence`); a run and a node's declaration are said
//! from their JSON (`run`); a noun's list is chosen and ordered by place
//! (`lists`).
//!
//! In `ffi/`, the one folder of this crate that may hold unsafe code
//! (ADR-0050, amendment 2026-11-16).
#![allow(unsafe_code)]

pub mod lists;
pub mod run;
pub mod said;
pub mod sentence;
