//! A module library, probed (ADR-0012, amendment 2026-11-17), for every
//! surface: opened and judged by the runtime loader's one rule
//! (`runtime::ffi::probed`), the one a node opens its modules with, and
//! said once — what it declares, its versions as text, and whether it
//! conforms. `xmip probe` renders it and `xmip_operate.h` section 26
//! answers with it.

use std::path::Path;

use runtime::ffi::probed::{self, Probed};
use serde_json::{Value, json};

/// What a probe found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleProbed {
    pub library: String,
    /// What the entrypoint returned; `XMIP_E_NOT_FOUND` where the library
    /// could not be asked.
    pub status: i32,
    pub provider: String,
    pub module: String,
    pub standard: String,
    pub abi_version: u32,
    /// `<major>.<minor>`; empty where the module made no instance.
    pub trait_version: String,
    /// `<major>.<minor>.<patch>`; empty where the module made no instance.
    pub module_version: String,
    pub last_error: String,
    /// Why the library could not be asked; empty where it was.
    pub unloadable: String,
    /// Why the host refuses it; empty where nothing is wrong.
    pub refusal: String,
    pub conforms: bool,
    /// What the module logged while it was held.
    pub log: Vec<String>,
}

impl ModuleProbed {
    /// Whether the library opened and its entrypoint was asked.
    #[must_use]
    pub fn loaded(&self) -> bool {
        self.unloadable.is_empty()
    }

    /// The probe as section 26's JSON: every member always there.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "library": self.library,
            "unloadable": self.unloadable,
            "status": self.status,
            "provider": self.provider,
            "module": self.module,
            "standard": self.standard,
            "abi_version": self.abi_version,
            "trait_version": self.trait_version,
            "module_version": self.module_version,
            "last_error": self.last_error,
            "refusal": self.refusal,
            "conforms": self.conforms,
            "log": self.log,
        })
    }
}

/// The library at `library`, probed.
#[must_use]
pub fn probe(library: &str) -> ModuleProbed {
    said(library, &probed::probe(Path::new(library)))
}

fn said(library: &str, probed: &Probed) -> ModuleProbed {
    let declared = probed.descriptor.as_ref();
    let text = |read: fn(&abi::ModuleDescriptor) -> String| declared.map(read).unwrap_or_default();

    ModuleProbed {
        library: library.to_string(),
        status: probed.status,
        provider: text(|d| d.provider.clone()),
        module: text(|d| d.module.clone()),
        standard: text(|d| d.standard.clone()),
        abi_version: declared.map_or(0, |d| d.abi_version),
        trait_version: text(|d| format!("{}.{}", d.trait_major, d.trait_minor)),
        module_version: text(|d| {
            format!("{}.{}.{}", d.module_major, d.module_minor, d.module_patch)
        }),
        last_error: probed.last_error.clone(),
        unloadable: probed.unloadable.clone(),
        refusal: probed.refusal.clone(),
        conforms: probed.conforms(),
        log: probed.log.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_library_that_is_not_there_is_unloadable_and_conforms_to_nothing() {
        let probed = probe("no_such_module_library.dll");

        assert!(!probed.loaded() && !probed.conforms);
        assert_eq!(probed.status, abi::ffi::status::NOT_FOUND);
        assert!(
            probed.unloadable.contains("would not open"),
            "{}",
            probed.unloadable
        );
        assert_eq!(probed.to_json()["provider"], "");
    }
}
