//! How a surface says things to a person, once (ADR-0052 clause 1): a mood
//! as a word, a rollup, an age, a figure, and the sentences every surface
//! says when there is nothing to say. A mood's word and a counted kind's
//! are `observe`'s; a stage's title is `node::Stage`'s.

use observe::{Counted, Health, HealthRecord};

use crate::surface::figures::{Figures, KINDS};
use crate::surface::scope_selection::grouped;
use crate::surface::scope_tree;

/// A mood as a word, lower case.
#[must_use]
pub fn mood(health: Health) -> &'static str {
    health.word()
}

/// A mood as a word, or `nothing recorded` where none is.
#[must_use]
pub fn mood_or_nothing(health: Option<Health>) -> &'static str {
    health.map_or("nothing recorded", mood)
}

/// A counted kind as a word.
#[must_use]
pub fn kind(counted: Counted) -> &'static str {
    counted.word()
}

/// What `records` roll up to, as a word.
#[must_use]
pub fn rollup(records: &[HealthRecord]) -> &'static str {
    mood_or_nothing(scope_tree::rollup(records))
}

/// A value, or a dash where it is empty or blank.
#[must_use]
pub fn value(value: &str) -> &str {
    if value.trim().is_empty() {
        "—"
    } else {
        value
    }
}

/// How long ago `observed` was at `now`, both Unix nanoseconds: seconds
/// under a minute, minutes under an hour, else hours; never negative.
#[must_use]
pub fn age(observed: i64, now: i64) -> String {
    let age = now.saturating_sub(observed).max(0);
    let seconds = age / 1_000_000_000;

    if seconds < 60 {
        format!("{seconds}s ago")
    } else if seconds < 3_600 {
        format!("{}m ago", seconds / 60)
    } else {
        format!("{}h ago", seconds / 3_600)
    }
}

/// A figure with its thousands grouped, or a dash where none was published.
#[must_use]
pub fn figure(value: Option<u64>) -> String {
    value.map_or_else(|| "–".to_string(), grouped)
}

/// The six figures, each named as a person reads it.
#[must_use]
pub fn figures(figures: &Figures) -> String {
    const NAMES: [&str; 6] = [
        "Streams", "Messages", "Journeys", "Bytes", "Retrying", "Failed",
    ];

    NAMES
        .iter()
        .zip(KINDS)
        .map(|(name, counted)| format!("{name} {}", figure(figures.of(counted))))
        .collect::<Vec<_>>()
        .join("  ")
}

/// Nothing is recorded at `scope` on the surface `source`.
#[must_use]
pub fn nothing_at(scope: &str, source: &str) -> String {
    format!("Nothing at {scope} ({source}).")
}

/// Nothing is beneath `scope` on the surface `source`.
#[must_use]
pub fn nothing_beneath(scope: &str, source: &str) -> String {
    format!("Nothing beneath {scope} ({source}).")
}

/// Nothing is measured at `scope` on the surface `source`.
#[must_use]
pub fn nothing_measured(scope: &str, source: &str) -> String {
    format!("Nothing measured at {scope} ({source}).")
}

/// Why a program that states no audit directory has no audit to read.
pub const NO_AUDIT_DIRECTORY: &str = "No audit directory is stated — neither \
    Xmip:AuditDirectory in this program's configuration nor XMIP_AUDIT_DIRECTORY — so its \
    records go to the operating system's log, and there is no audit here to read.";

/// What follows a hidden run's record or group: ` · test`, else nothing.
#[must_use]
pub const fn test(hidden: bool) -> &'static str {
    if hidden { " · test" } else { "" }
}

/// An RFC 3339 time without its fractional seconds, its zone kept.
#[must_use]
pub fn to_the_second(at: &str) -> String {
    let bytes = at.as_bytes();

    if bytes.len() <= 19 || bytes[19] != b'.' {
        return at.to_string();
    }

    let zone = 20
        + bytes[20..]
            .iter()
            .take_while(|byte| byte.is_ascii_digit())
            .count();
    format!("{}{}", &at[..19], &at[zone..])
}

/// A stage's title, `Work Process`; the word itself where it is no stage.
#[must_use]
pub fn stage(stage: &str) -> String {
    node::Stage::named(stage).map_or_else(|| stage.to_string(), |stage| stage.title().to_string())
}

/// What a process stage holding Subscriptions alone says.
pub const NO_WORK_PROCESS: &str = "no Work Process placed";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_age_is_seconds_then_minutes_then_hours_and_never_negative() {
        let second = 1_000_000_000;

        assert_eq!(age(0, 59 * second), "59s ago");
        assert_eq!(age(0, 61 * second), "1m ago");
        assert_eq!(age(0, 3_700 * second), "1h ago");
        assert_eq!(age(10 * second, 0), "0s ago");
    }

    #[test]
    fn a_figure_is_grouped_or_a_dash_and_a_time_loses_its_fraction() {
        assert_eq!(figure(Some(12_345)), "12,345");
        assert_eq!(figure(None), "–");
        assert_eq!(
            to_the_second("2026-09-11T12:00:00.1234567+00:00"),
            "2026-09-11T12:00:00+00:00"
        );
        assert_eq!(
            to_the_second("2026-09-11T12:00:00Z"),
            "2026-09-11T12:00:00Z"
        );
        assert_eq!(value(" "), "—");
    }
}
