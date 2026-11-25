//! A noun's list chosen and ordered as a query asks, answered by place in
//! the list a surface handed over — the very JSON sections 14, 15 and 16
//! hand it — and the words a row of it shows: `operate::surface`'s three
//! queries and `list_order`, the one order every surface sorts by.
#![allow(unsafe_code)]

use abi::ffi::{Str, status};
use abi::operate::words::{list, list_word};
use observe::{
    DeadMessage, EventSubscription, PauseState, Subscription, Unheard, WorkProcessInstances,
};
use serde_json::{Value, json};

use crate::ffi::rule::fill;
use crate::ffi::text::answer;
use crate::surface::dead_message_query::{self, DeadMessageQuery};
use crate::surface::event_subscription_query::{self, EventSubscriptionQuery};
use crate::surface::instance_query::{self, InstanceQuery};
use crate::surface::subscription_query::{self, SubscriptionQuery};

/// `xmip_list_query_v1`: the list `entries` (`XMIP_LIST_*`, its section's
/// JSON) chosen and ordered as `query` asks — `{"pattern", "location",
/// "name" | "id" | "message", "sort", "order"}` — answered as
/// `{"places": [...]}`, each a place in the list, and for Event
/// subscriptions `"unheard"`, the places of the members not heard that
/// stand there; `XMIP_E_INVALID` with the REFUSED sentence of a column or
/// an order the list does not take.
///
/// # Safety
/// Both texts readable for their lengths; `out` has room for `cap` bytes;
/// `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_list_query_v1(
    which: i32,
    entries: Str,
    query: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(entries, query, out, cap, out_len, |entries, query| {
            let entries: Value = serde_json::from_str(entries)
                .map_err(|failure| format!("the list is not JSON: {failure}"))?;
            let query: Value = serde_json::from_str(if query.is_empty() { "{}" } else { query })
                .map_err(|failure| format!("the query is not JSON: {failure}"))?;
            chosen(which, &entries, &query).map(|answer| answer.to_string())
        })
    }
}

fn chosen(which: i32, entries: &Value, query: &Value) -> Result<Value, String> {
    let text = |key: &str| query[key].as_str().map(str::to_string);
    match which {
        list::SUBSCRIPTIONS => {
            let asked = SubscriptionQuery {
                pattern: text("pattern"),
                location: text("location"),
                name: text("name"),
                sort: text("sort"),
                order: text("order"),
            };
            Ok(json!({ "places": asked.places(&each(entries, "subscriptions", subscription))? }))
        }
        list::EVENT_SUBSCRIPTIONS => {
            let asked = EventSubscriptionQuery {
                pattern: text("pattern"),
                location: text("location"),
                id: query["id"].as_u64(),
                sort: text("sort"),
                order: text("order"),
            };
            let subscriptions = each(entries, "event_subscriptions", event_subscription);
            Ok(json!({
                "places": asked.places(&subscriptions)?,
                "unheard": asked.unheard_places(&each(entries, "unheard", unheard)),
            }))
        }
        list::DEAD_MESSAGES => {
            let asked = DeadMessageQuery {
                pattern: text("pattern"),
                location: text("location"),
                message: text("message"),
                sort: text("sort"),
                order: text("order"),
            };
            Ok(json!({ "places": asked.places(&each(entries, "dead_messages", dead_message))? }))
        }
        list::INSTANCES => {
            let asked = InstanceQuery {
                location: text("location"),
                work_process: text("work_process"),
                instance: text("instance"),
            };
            // Each entry keeps its place: one that does not read is read as
            // an empty one, never dropped, so no place after it shifts.
            let lists: Vec<WorkProcessInstances> = entries["work_process_instances"]
                .as_array()
                .map(|lists| {
                    lists
                        .iter()
                        .map(|list| serde_json::from_value(list.clone()).unwrap_or_default())
                        .collect()
                })
                .unwrap_or_default();
            let (places, instances): (Vec<usize>, Vec<Vec<usize>>) =
                asked.places(&lists).into_iter().unzip();
            Ok(json!({ "places": places, "instances": instances }))
        }
        _ => Err(format!("{which} is no list of section 28.3")),
    }
}

/// `xmip_list_columns_v1`: the columns the list `which` sorts by, the first
/// its default, static.
///
/// # Safety
/// `out` has room for `cap` strings; `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_list_columns_v1(
    which: i32,
    out: *mut Str,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    abi::guarded(status::PANIC, || {
        let columns: &[&str] = match which {
            list::SUBSCRIPTIONS => &subscription_query::COLUMNS,
            list::EVENT_SUBSCRIPTIONS => &event_subscription_query::COLUMNS,
            list::DEAD_MESSAGES => &dead_message_query::COLUMNS,
            list::INSTANCES => &instance_query::COLUMNS,
            _ => return status::INVALID,
        };
        // SAFETY: per the contract above.
        unsafe { fill(columns.iter().copied(), out, cap, out_len) };
        status::OK
    })
}

/// `xmip_list_words_v1`: the word `which` (`XMIP_LIST_WORD_*`) a row shows,
/// from its two texts: a Subscription's scope from its node and name, a
/// Dead Message Queue entry's from its node and Message, who an Event
/// subscriber is from its declared name and its Party, and a member not
/// heard from the node that does not hear it and what it says.
///
/// # Safety
/// Both texts readable for their lengths; `out` has room for `cap` bytes;
/// `out_len` is writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn xmip_list_words_v1(
    which: i32,
    first: Str,
    second: Str,
    out: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: the caller's contract is `answer`'s, passed on whole.
    unsafe {
        answer(
            first,
            second,
            out,
            cap,
            out_len,
            |first, second| match which {
                list_word::SUBSCRIPTION_SCOPE => Ok(subscription_query::scope(&Subscription {
                    node: first.to_string(),
                    name: second.to_string(),
                    ..Subscription::default()
                })),
                list_word::DEAD_MESSAGE_SCOPE => Ok(dead_message_query::scope(&DeadMessage {
                    node: first.to_string(),
                    message: second.to_string(),
                    ..DeadMessage::default()
                })),
                list_word::EVENT_WHO => Ok(event_subscription_query::who(&EventSubscription {
                    subscriber: first.to_string(),
                    party: second.to_string(),
                    ..EventSubscription::default()
                })
                .to_string()),
                list_word::UNHEARD_LINE => Ok(event_subscription_query::said_by(first, second)),
                _ => Err(format!("{which} is no word of section 28.3")),
            },
        )
    }
}

// The entries of the array `key` of `list`, each read by `read`.
fn each<T>(list: &Value, key: &str, read: fn(&Value) -> T) -> Vec<T> {
    list[key]
        .as_array()
        .map(|entries| entries.iter().map(read).collect())
        .unwrap_or_default()
}

fn text(entry: &Value, key: &str) -> String {
    entry[key].as_str().unwrap_or_default().to_string()
}

fn number(entry: &Value, key: &str) -> u64 {
    entry[key].as_u64().unwrap_or(0)
}

fn moment(entry: &Value, key: &str) -> i64 {
    entry[key].as_i64().unwrap_or(0)
}

fn subscription(entry: &Value) -> Subscription {
    Subscription {
        node: text(entry, "node"),
        name: text(entry, "name"),
        application: text(entry, "application"),
        filter: text(entry, "filter"),
        destination: text(entry, "destination"),
        send_ports: entry["send_ports"]
            .as_array()
            .map(|ports| {
                ports
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        file: text(entry, "file"),
        configuration: text(entry, "configuration"),
        state: PauseState::read(&text(entry, "state")),
        by: text(entry, "by"),
        picked_up: number(entry, "picked_up"),
        held: number(entry, "held"),
        since_unix_nanos: moment(entry, "since_unix_nanos"),
    }
}

fn event_subscription(entry: &Value) -> EventSubscription {
    EventSubscription {
        node: text(entry, "node"),
        id: number(entry, "id"),
        subscriber: text(entry, "subscriber"),
        party: text(entry, "party"),
        action: text(entry, "action"),
        scope: text(entry, "scope"),
        state: PauseState::read(&text(entry, "state")),
        queued: number(entry, "queued"),
        capacity: number(entry, "capacity"),
        delivered: number(entry, "delivered"),
        missed: number(entry, "missed"),
        since_unix_nanos: moment(entry, "since_unix_nanos"),
    }
}

fn unheard(entry: &Value) -> Unheard {
    Unheard {
        by: text(entry, "by"),
        node: text(entry, "node"),
        since_unix_nanos: moment(entry, "since_unix_nanos"),
        why: text(entry, "why"),
    }
}

fn dead_message(entry: &Value) -> DeadMessage {
    let pairs = |key: &str| -> Vec<[String; 2]> {
        entry[key]
            .as_array()
            .map(|pairs| {
                pairs
                    .iter()
                    .map(|pair| [text_at(pair, 0), text_at(pair, 1)])
                    .collect()
            })
            .unwrap_or_default()
    };
    DeadMessage {
        node: text(entry, "node"),
        message: text(entry, "message"),
        sequence: number(entry, "sequence"),
        location: text(entry, "location"),
        received_unix_nanos: moment(entry, "received_unix_nanos"),
        validation: pairs("validation"),
        promoted: pairs("promoted"),
        declines: pairs("declines"),
    }
}

fn text_at(pair: &Value, at: usize) -> String {
    pair[at].as_str().unwrap_or_default().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::operate::words::{ListColumnsFn, ListQueryFn, SentenceFn};

    #[test]
    fn the_exports_have_the_shapes_the_binding_declares() {
        let _: ListQueryFn = xmip_list_query_v1;
        let _: ListColumnsFn = xmip_list_columns_v1;
        let _: SentenceFn = xmip_list_words_v1;
    }

    #[test]
    fn a_list_is_chosen_and_ordered_by_place_or_refused_in_words() {
        let test = configure::fixture::test_cluster();
        let (a, b) = (test.node_scope(0), test.node_scope(1));
        let entries = json!({ "subscriptions": [
            { "node": b, "name": "Orders", "state": "active", "held": 1 },
            { "node": a, "name": "billing", "state": "paused", "held": 9 },
            { "node": a, "name": "Audit", "state": "active", "held": 2 },
        ]});

        let ordinal = chosen(list::SUBSCRIPTIONS, &entries, &json!({})).expect("chosen");
        assert_eq!(
            ordinal["places"],
            json!([2, 0, 1]),
            "upper case first, ordinally"
        );

        let held = json!({ "sort": "held", "order": "descending", "location": a });
        let held = chosen(list::SUBSCRIPTIONS, &entries, &held).expect("chosen");
        assert_eq!(held["places"], json!([1, 2]));

        let refused = chosen(list::SUBSCRIPTIONS, &entries, &json!({ "sort": "colour" }));
        assert_eq!(
            refused,
            Err(format!(
                "REFUSED: 'colour' is no column; the columns are {}.",
                subscription_query::COLUMNS.join(", ")
            ))
        );
    }
}
