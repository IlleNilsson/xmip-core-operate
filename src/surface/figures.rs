//! The six measurements at a scope, said in one order by every surface:
//! Streams, Messages, Journeys, bytes (ADR-0027 clause 5), Retrying and
//! Failed. A figure nothing published is `None`, never a zero.

use observe::Counted;

use crate::surface::OperatorSurface;

/// Every counted kind, in the order a surface says them.
pub const KINDS: [Counted; 6] = [
    Counted::Streams,
    Counted::Messages,
    Counted::Journeys,
    Counted::Bytes,
    Counted::Retrying,
    Counted::Failed,
];

/// The six figures at `scope` and when the newest was observed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Figures {
    pub scope: String,
    /// Each kind's figure, in [`KINDS`] order.
    pub values: [Option<u64>; 6],
    /// When the newest figure was observed, Unix nanoseconds.
    pub observed: Option<i64>,
}

impl Figures {
    /// Nothing published at `scope`.
    #[must_use]
    pub fn none(scope: &str) -> Self {
        Self {
            scope: scope.to_string(),
            ..Self::default()
        }
    }

    /// The figures `surface` measures at `scope`, kind by kind.
    #[must_use]
    pub fn read(surface: &(impl OperatorSurface + ?Sized), scope: &str) -> Self {
        let mut figures = Self::none(scope);

        for (at, counted) in KINDS.into_iter().enumerate() {
            if let Some(count) = surface.measure(scope, counted) {
                figures.values[at] = Some(count.value);
                figures.observed = figures.observed.max(Some(count.observed_unix_nanos));
            }
        }

        figures
    }

    /// `parts` summed at `scope`: a kind none of them published stays
    /// `None`.
    #[must_use]
    pub fn sum<'a>(scope: &str, parts: impl IntoIterator<Item = &'a Self>) -> Self {
        let mut sum = Self::none(scope);

        for part in parts {
            for (total, value) in sum.values.iter_mut().zip(part.values) {
                if let Some(value) = value {
                    *total = Some(total.unwrap_or(0).wrapping_add(value));
                }
            }
            sum.observed = sum.observed.max(part.observed);
        }

        sum
    }

    /// The figure of `counted`.
    #[must_use]
    pub fn of(&self, counted: Counted) -> Option<u64> {
        KINDS
            .iter()
            .position(|&kind| kind == counted)
            .and_then(|at| self.values[at])
    }

    /// Whether any figure was published.
    #[must_use]
    pub fn has_values(&self) -> bool {
        self.values.iter().any(Option::is_some)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sum_keeps_a_kind_nothing_published_as_nothing() {
        let mut one = Figures::none("a");
        one.values[0] = Some(2);
        one.observed = Some(5);
        let mut two = Figures::none("b");
        two.values[0] = Some(3);
        two.values[5] = Some(1);

        let sum = Figures::sum("c", [&one, &two]);

        assert_eq!(sum.of(Counted::Streams), Some(5));
        assert_eq!(sum.of(Counted::Failed), Some(1));
        assert_eq!(sum.of(Counted::Bytes), None);
        assert_eq!(sum.observed, Some(5));
        assert!(sum.has_values() && !Figures::none("d").has_values());
    }
}
