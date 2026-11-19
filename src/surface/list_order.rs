//! How a list of one of a node's nouns is ordered, once for every list and
//! every surface: by the column `--sort` names, its first where none is,
//! ascending unless `--order descending`, a text compared ordinally — the
//! one order every surface sorts by, so no two disagree — then by node and
//! by what names the entry on it. A column or an order the list does not
//! take is refused in words.

/// What one entry is ordered by in one column.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Text(String),
    Number(u64),
    /// A moment, Unix nanoseconds, compared to the tenth of a microsecond.
    Moment(i64),
}

impl Key {
    /// A moment's key, compared to the tick as every surface has dated it.
    #[must_use]
    pub const fn moment(unix_nanos: i64) -> Self {
        Self::Moment(unix_nanos / 100)
    }
}

/// The column and direction a line asks for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListOrder {
    pub column: String,
    pub descending: bool,
}

impl ListOrder {
    /// The order `sort` and `order` ask of a list whose columns are
    /// `columns`, its first the default.
    ///
    /// # Errors
    ///
    /// A column or an order the list does not take: the REFUSED sentence.
    pub fn asked(
        sort: Option<&str>,
        order: Option<&str>,
        columns: &[&str],
    ) -> Result<Self, String> {
        let column = sort.unwrap_or(columns[0]);

        if !columns.contains(&column) {
            return Err(format!(
                "REFUSED: '{column}' is no column; the columns are {}.",
                columns.join(", ")
            ));
        }
        if !matches!(order, None | Some("ascending" | "descending")) {
            return Err(format!(
                "REFUSED: '{}' is no order; the orders are ascending, descending.",
                order.unwrap_or_default()
            ));
        }

        Ok(Self {
            column: column.to_string(),
            descending: order == Some("descending"),
        })
    }

    /// The places in `entries` of those `chosen` takes, in this order: each
    /// by `key` in the column, then by `then`, ascending, ties kept as they
    /// came. A surface across the C ABI keeps the entries it read and is
    /// answered by place (`ffi::words`).
    pub fn places<T, K: Ord>(
        &self,
        entries: &[T],
        chosen: impl Fn(&T) -> bool,
        key: impl Fn(&T, &str) -> Key,
        then: impl Fn(&T) -> K,
    ) -> Vec<usize> {
        let mut places: Vec<usize> = (0..entries.len())
            .filter(|&at| chosen(&entries[at]))
            .collect();
        places.sort_by(|&a, &b| {
            let (a, b) = (&entries[a], &entries[b]);
            let by = key(a, &self.column).cmp(&key(b, &self.column));
            let by = if self.descending { by.reverse() } else { by };
            by.then_with(|| then(a).cmp(&then(b)))
        });
        places
    }
}

/// Whether `entry` stands where `location` names, at or beneath it; every
/// entry where none is named.
#[must_use]
pub fn placed(entry: &str, location: Option<&str>) -> bool {
    location
        .filter(|location| !location.is_empty())
        .is_none_or(|location| crate::surface::scope_tree::beneath(entry, location))
}

/// Whether any of `candidates` matches `pattern`; every one where none is
/// named (`node::wildcard`).
#[must_use]
pub fn patterned(candidates: &[&str], pattern: Option<&str>) -> bool {
    pattern
        .filter(|pattern| !pattern.is_empty())
        .is_none_or(|pattern| {
            candidates
                .iter()
                .any(|candidate| node::wildcard::matches(candidate, pattern))
        })
}

/// The cluster a node's scope is in: its first segment, empty for none.
#[must_use]
pub fn cluster(node: &str) -> String {
    crate::surface::scope_tree::parts(node)
        .first()
        .map_or_else(String::new, |cluster| (*cluster).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_order_is_a_column_and_a_direction_or_refused_in_words() {
        let columns = ["one", "two"];

        assert_eq!(
            ListOrder::asked(None, None, &columns),
            Ok(ListOrder {
                column: "one".into(),
                descending: false
            })
        );
        assert_eq!(
            ListOrder::asked(Some("three"), None, &columns),
            Err("REFUSED: 'three' is no column; the columns are one, two.".into())
        );
        assert_eq!(
            ListOrder::asked(None, Some("up"), &columns),
            Err("REFUSED: 'up' is no order; the orders are ascending, descending.".into())
        );
    }

    #[test]
    fn a_sort_is_by_the_column_then_by_what_names_each_ascending() {
        let order = ListOrder::asked(Some("n"), Some("descending"), &["n"]).expect("an order");
        let entries = [("b", 1), ("a", 2), ("c", 1), ("d", 9)];

        let places = order.places(
            &entries,
            |entry| entry.0 != "d",
            |entry, _| Key::Number(entry.1),
            |entry| entry.0,
        );

        assert_eq!(places, [1, 0, 2], "a 2, then b 1 and c 1; d not chosen");
        let test = configure::fixture::test_cluster();
        let (at, node) = (test.node_scope(0), &test.node(0).name);
        let other = format!("xmip:///{}", test.other());

        assert!(placed(&at, Some(&test.scope())));
        assert!(!placed(&at, Some(&other)));
        assert!(patterned(&[at.as_str()], Some(&format!("*/{node}"))));
        assert_eq!(cluster(&at), test.name);
    }
}
