//! What came of an operator's act, said once for every surface: taken by
//! the node it is asked of, asked — written to Xmip Storage and not taken
//! yet within the door's bound, its node taking it as soon as it runs — or
//! refused (ADR-0014, amendment 2026-11-13); what came of a pause or a
//! resume of a scope; and what came of setting a secret.

use runtime::plane::remote::Outcome;

/// What came of an act.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActOutcome {
    Taken,
    Asked,
    Refused,
}

impl ActOutcome {
    /// The outcome of what a node's door answered: taken, asked, and
    /// refused for every other.
    #[must_use]
    pub const fn of(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Taken => Self::Taken,
            Outcome::Asked => Self::Asked,
            _ => Self::Refused,
        }
    }

    /// The outcome of the status an act's export answered with
    /// (`xmip_remote_act_v1`): `XMIP_OK` taken, `XMIP_E_AGAIN` asked, and
    /// refused for every other.
    #[must_use]
    pub const fn of_status(code: i32) -> Self {
        match code {
            abi::ffi::status::OK => Self::Taken,
            abi::ffi::status::AGAIN => Self::Asked,
            _ => Self::Refused,
        }
    }

    /// Its word, lower case: `taken`, `asked`, `refused`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Taken => "taken",
            Self::Asked => "asked",
            Self::Refused => "refused",
        }
    }

    /// The sentence a person reads: `OK.`, `ASKED.` or `REFUSED:`, then
    /// `result`.
    #[must_use]
    pub fn said(self, result: &str) -> String {
        match self {
            Self::Taken => format!("OK. {result}"),
            Self::Asked => format!("ASKED. {result}"),
            Self::Refused => format!("REFUSED: {result}"),
        }
    }
}

/// An act on one of a node's nouns and what came of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Acted {
    pub outcome: ActOutcome,
    /// The node's sentence, or why it was not asked.
    pub said: String,
}

impl Acted {
    /// Not asked of any node, for `why`.
    #[must_use]
    pub fn declined(why: impl Into<String>) -> Self {
        Self {
            outcome: ActOutcome::Refused,
            said: why.into(),
        }
    }

    /// Whether the node took it.
    #[must_use]
    pub fn applied(&self) -> bool {
        self.outcome == ActOutcome::Taken
    }
}

/// The two acts an operator takes on a scope (ADR-0027 clause 5): there is
/// no start, stop or restart, because the thing that watches must not be
/// able to stop the thing it watches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeAction {
    Pause,
    Resume,
}

impl ScopeAction {
    /// Its word, as a document writes it: `Pause`, `Resume`.
    #[must_use]
    pub const fn word(self) -> &'static str {
        match self {
            Self::Pause => "Pause",
            Self::Resume => "Resume",
        }
    }
}

/// What came of a pause or a resume of a scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeOperation {
    pub scope: String,
    pub action: ScopeAction,
    pub applied: bool,
    pub result: String,
}

/// Who acts: `stated` trimmed, else `user`, the account the process runs
/// as.
#[must_use]
pub fn who(stated: Option<&str>, user: &str) -> String {
    match stated.map(str::trim) {
        Some(named) if !named.is_empty() => named.to_string(),
        _ => user.to_string(),
    }
}

/// A secret a node keeps, as a surface lists it: its name, its kind —
/// password, key or certificate — and when and by whom it was last set
/// (ADR-0019, amendment 2026-10-29). Never its value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretRecord {
    pub name: String,
    pub kind: String,
    /// When it was last set, Unix nanoseconds.
    pub set_unix_nanos: u64,
    pub set_by: String,
}

impl SecretRecord {
    /// The kinds a secret is.
    pub const KINDS: [&'static str; 3] = ["password", "key", "certificate"];

    /// The secrets a node's answer lists; none where it lists none.
    #[must_use]
    pub fn listed(answer: &serde_json::Value) -> Vec<Self> {
        let text = |secret: &serde_json::Value, name: &str| {
            secret[name].as_str().unwrap_or_default().to_string()
        };

        answer["secrets"]
            .as_array()
            .map(|secrets| {
                secrets
                    .iter()
                    .map(|secret| Self {
                        name: text(secret, "name"),
                        kind: text(secret, "kind"),
                        set_unix_nanos: secret["set_unix_nanos"].as_u64().unwrap_or(0),
                        set_by: text(secret, "set_by"),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// What came of setting a secret: whether it was set, and the sentence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretOperation {
    pub node: String,
    pub name: String,
    pub applied: bool,
    pub result: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_outcome_is_said_in_one_of_three_words() {
        assert_eq!(ActOutcome::of(Outcome::Taken).said("done"), "OK. done");
        assert_eq!(ActOutcome::of(Outcome::Asked).said("later"), "ASKED. later");
        assert_eq!(ActOutcome::of(Outcome::Denied).said("no"), "REFUSED: no");
        assert_eq!(ActOutcome::Refused.word(), "refused");
        assert_eq!(who(Some(" ann "), "me"), "ann");
        assert_eq!(who(Some(" "), "me"), "me");
        assert_eq!(who(None, "me"), "me");
    }
}
