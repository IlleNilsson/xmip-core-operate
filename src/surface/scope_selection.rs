//! What a scope argument selects: the one scope an operator spelled, or the
//! several a wildcard names among the scopes that exist (ADR-0059 clauses 7
//! and 8, read for every surface). The match itself is `node::wildcard`'s;
//! which of the matches stand — the topmost, none beneath another — and the
//! refusal of a pattern that matches nothing are written here once.

use crate::surface::OperatorSurface;
use crate::surface::scope_index::ScopeIndex;
use crate::surface::scope_tree::{self, ROOT};

/// What an argument selected.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeSelection {
    /// The argument as given, or the surface's root where none was.
    pub argument: String,
    /// Whether it was a wildcard.
    pub patterned: bool,
    /// The scopes it names: the one spelled, or the topmost matches.
    pub scopes: Vec<String>,
}

impl ScopeSelection {
    /// `scope`, exactly as spelled.
    #[must_use]
    pub fn exactly(scope: &str) -> Self {
        Self {
            argument: scope.to_string(),
            patterned: false,
            scopes: vec![scope.to_string()],
        }
    }

    /// What `argument` selects on `surface`: empty is the surface's root.
    ///
    /// # Errors
    ///
    /// A wildcard that matches no scope here: the refusal, what is beneath
    /// the literal part of the pattern, and the surface's source.
    pub fn of(surface: &(impl OperatorSurface + ?Sized), argument: &str) -> Result<Self, String> {
        let pattern = if argument.is_empty() {
            surface.root()
        } else {
            argument.to_string()
        };

        if !has_wildcard(&pattern) {
            return Ok(Self::exactly(&pattern));
        }

        Self::over(&surface.index(), &pattern, &surface.source())
    }

    /// What `pattern` selects among the scopes `index` holds: itself where
    /// it holds no wildcard, else the topmost it matches.
    ///
    /// # Errors
    ///
    /// A wildcard that matches no scope there: the refusal, what is beneath
    /// the literal part of the pattern, and `source`.
    pub fn over(index: &ScopeIndex, pattern: &str, source: &str) -> Result<Self, String> {
        if !has_wildcard(pattern) {
            return Ok(Self::exactly(pattern));
        }

        let chosen = topmost(index.matching(pattern));

        if chosen.is_empty() {
            return Err(refusal(index, pattern, source));
        }

        Ok(Self {
            argument: pattern.to_string(),
            patterned: true,
            scopes: chosen,
        })
    }
}

/// Whether `pattern` holds a `*` or a `?`.
#[must_use]
pub fn has_wildcard(pattern: &str) -> bool {
    pattern.contains(['*', '?'])
}

/// `scope` as the tree spells it.
#[must_use]
pub fn normal(scope: &str) -> String {
    scope_tree::join(&scope_tree::parts(scope))
}

/// The scopes none of the others contains, in ordinal order.
#[must_use]
pub fn topmost(scopes: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut ordered: Vec<String> = scopes.into_iter().map(|scope| normal(&scope)).collect();
    ordered.sort();
    ordered.dedup();

    let mut kept: Vec<String> = Vec::new();

    for scope in ordered {
        // Against every one kept: ordinal order puts `a-x` between `a` and
        // `a/b`, so the last kept is not always the ancestor to ask.
        if !kept.iter().any(|above| scope_tree::beneath(&scope, above)) {
            kept.push(scope);
        }
    }

    kept
}

// The refusal of a pattern that names nothing: what is beneath its literal
// part, a dozen at most, or how many scopes are published.
fn refusal(index: &ScopeIndex, pattern: &str, source: &str) -> String {
    const MOST: usize = 12;
    let under = normal(literal(pattern));
    let beneath = index.branches(&under);
    let mut labels: Vec<&str> = beneath.iter().map(|branch| branch.label.as_str()).collect();
    labels.sort_unstable();

    let named = labels
        .iter()
        .take(MOST)
        .copied()
        .collect::<Vec<_>>()
        .join(", ");
    let rest = if beneath.len() > MOST {
        format!(" and {} more", grouped(count(beneath.len() - MOST)))
    } else {
        String::new()
    };
    let what = if beneath.is_empty() {
        format!("{} scope(s) published", grouped(count(index.scopes())))
    } else {
        format!("beneath {under} there is: {named}{rest}")
    };

    format!(
        "REFUSED  no scope matches {pattern}\n         {what}\n         source \
         {source}"
    )
}

fn count(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

// The part of a pattern before the segment its first wildcard is in.
fn literal(pattern: &str) -> &str {
    let Some(wild) = pattern.find(['*', '?']) else {
        return pattern;
    };

    match pattern[..wild].rfind('/') {
        Some(slash) if slash > 0 => &pattern[..slash],
        _ => ROOT,
    }
}

/// A count with a comma between each group of three digits, `1,234`.
#[must_use]
pub fn grouped(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);

    for (at, digit) in digits.chars().enumerate() {
        if at > 0 && (digits.len() - at).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_topmost_of_matches_stand_and_none_beneath_another() {
        let test = configure::fixture::test_cluster();
        let (a, b) = (test.node_scope(0), test.node_scope(1));
        let beside = format!("{a}-x");
        let scopes = [format!("{a}/receive"), a.clone(), beside.clone(), a.clone()];

        assert_eq!(topmost(scopes), [a.clone(), beside]);
        // What is beneath another goes, whatever order it came in; the root
        // swallows everything beneath it.
        let beneath = [
            format!("{a}/receive"),
            a.clone(),
            format!("{a}/receive/tcp"),
            b.clone(),
        ];
        let mut both = [a.clone(), b];
        both.sort();
        assert_eq!(topmost(beneath), both);
        let root = [ROOT.to_string(), test.scope(), a];
        assert_eq!(topmost(root), [ROOT]);
    }

    #[test]
    fn a_pattern_s_literal_part_is_before_its_wild_segment() {
        let cluster = configure::fixture::test_cluster().scope();
        assert_eq!(
            literal(&format!("{cluster}/node/*/send")),
            format!("{cluster}/node")
        );
        assert_eq!(normal(literal(&format!("{cluster}*"))), ROOT);
        assert_eq!(literal("plain"), "plain");
        assert!(has_wildcard("a?") && !has_wildcard("a"));
        assert_eq!(grouped(1_234_567), "1,234,567");
        assert_eq!(grouped(12), "12");
    }
}
