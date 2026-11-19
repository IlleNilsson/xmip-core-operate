//! What came of handing the runtime a node configuration to validate: the
//! document, the status, the problems the runtime listed, and the sentence
//! every surface says about it (ADR-0052 clause 4). A surface decides by the
//! record, never by the sentence. The problems are
//! `runtime::start::validate`'s.

use abi::ffi::status;

/// One verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationVerdict {
    /// The document, as named.
    pub path: String,
    /// The header's status: `XMIP_OK`, or `XMIP_E_INVALID` with problems.
    pub status: i32,
    pub problems: Vec<String>,
    /// The sentence.
    pub said: String,
}

impl ConfigurationVerdict {
    /// The verdict on `path` the runtime gave as `status` and `problems`.
    #[must_use]
    pub fn validated(path: &str, status: i32, problems: Vec<String>) -> Self {
        let said = if status == status::OK && problems.is_empty() {
            format!("{path} is valid")
        } else if status == status::UNSUPPORTED {
            format!(
                "this runtime does not export {}",
                abi::operate::XMIP_VALIDATE_ENTRYPOINT
            )
        } else {
            let why = if problems.is_empty() {
                abi::status_meaning::explain(status).to_string()
            } else {
                problems.join("; ")
            };
            format!("{path} is invalid: {why}")
        };

        Self {
            path: path.to_string(),
            status,
            problems,
            said,
        }
    }

    /// The verdict on `path` from the problems the runtime listed: valid
    /// where it listed none.
    #[must_use]
    pub fn of(path: &str, problems: Vec<String>) -> Self {
        let status = if problems.is_empty() {
            status::OK
        } else {
            status::INVALID
        };
        Self::validated(path, status, problems)
    }

    /// Whether the document is valid: by the record, never the sentence.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.status == status::OK && self.problems.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_verdict_says_valid_or_its_problems_and_decides_by_its_record() {
        let valid = ConfigurationVerdict::of("node.xmip", Vec::new());
        assert!(valid.ok());
        assert_eq!(valid.said, "node.xmip is valid");

        let invalid = ConfigurationVerdict::of(
            "node.xmip",
            vec!["no name".to_string(), "no receive location".to_string()],
        );
        assert!(!invalid.ok());
        assert_eq!(invalid.status, status::INVALID);
        assert_eq!(
            invalid.said,
            "node.xmip is invalid: no name; no receive location"
        );

        let bare = ConfigurationVerdict::validated("node.xmip", status::INVALID, Vec::new());
        assert_eq!(
            bare.said,
            "node.xmip is invalid: an argument was outside its contract"
        );
    }
}
