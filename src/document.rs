//! A configuration document read, written and one key of it set, as
//! `xmip_operate.h` section 27 hands them across: the document as JSON, in
//! memory only (ADR-0031 clause 2). Reading, writing and editing are
//! `configure::document`'s, the one the runtime reads with; this turns its
//! tables into JSON and back and nothing else. In a remote build too: the
//! phone reads its own document (ADR-0031, amendment 2026-11-17).

use serde_json::{Map, Number, Value as Json};
use toml::{Table, Value};

/// `text`, TOML, as a JSON object: tables as objects, arrays as arrays, a
/// date or time as its TOML text.
///
/// # Errors
/// The text is not TOML, in the reader's words.
pub fn read(text: &str) -> Result<String, String> {
    let table = configure::document::read(text)?;
    Ok(Json::Object(object(&table)).to_string())
}

/// `json`, a JSON object, written as TOML.
///
/// # Errors
/// It is not a JSON object, it holds a null, or TOML cannot write it.
pub fn write(json: &str) -> Result<String, String> {
    configure::document::write(&table_of(json)?)
}

/// `json`, a JSON object, as the TOML table it writes: what a host's own
/// reader read of its document, handed across (section 28.2).
///
/// # Errors
/// It is not a JSON object, or it holds a null.
pub(crate) fn table_of(json: &str) -> Result<Table, String> {
    let Json::Object(document) = parsed(json)? else {
        return Err("a document is a JSON object".to_string());
    };
    table(&document)
}

/// `text` with one key set or removed, as `edit` says:
/// `{"table":[...],"key":"...","value":...}`, `value` absent or null to
/// remove the key.
///
/// # Errors
/// The edit is not that object, its value holds a null, or
/// `configure::document::set` refuses the text.
pub fn set(text: &str, edit: &str) -> Result<String, String> {
    let Json::Object(edit) = parsed(edit)? else {
        return Err("an edit is a JSON object".to_string());
    };
    let names = match edit.get("table") {
        None => Vec::new(),
        Some(Json::Array(names)) => names
            .iter()
            .map(|name| name.as_str().map(str::to_string))
            .collect::<Option<Vec<_>>>()
            .ok_or("an edit's table is its keys, each a string")?,
        Some(_) => return Err("an edit's table is an array of keys".to_string()),
    };
    let Some(Json::String(key)) = edit.get("key") else {
        return Err("an edit names its key, a string".to_string());
    };
    let value = match edit.get("value") {
        None | Some(Json::Null) => None,
        Some(given) => Some(value(given)?),
    };

    configure::document::set(text, &names, key, value.as_ref())
}

fn parsed(json: &str) -> Result<Json, String> {
    serde_json::from_str(json).map_err(|error| format!("not JSON: {error}"))
}

fn object(table: &Table) -> Map<String, Json> {
    table
        .iter()
        .map(|(key, value)| (key.clone(), json(value)))
        .collect()
}

fn json(value: &Value) -> Json {
    match value {
        Value::String(text) => Json::String(text.clone()),
        Value::Integer(number) => Json::Number(Number::from(*number)),
        // JSON has no NaN or infinity; TOML's text of one says which.
        Value::Float(number) => {
            Number::from_f64(*number).map_or_else(|| Json::String(value.to_string()), Json::Number)
        }
        Value::Boolean(flag) => Json::Bool(*flag),
        Value::Datetime(moment) => Json::String(moment.to_string()),
        Value::Array(items) => Json::Array(items.iter().map(json).collect()),
        Value::Table(table) => Json::Object(object(table)),
    }
}

fn table(object: &Map<String, Json>) -> Result<Table, String> {
    object
        .iter()
        .map(|(key, given)| Ok((key.clone(), value(given)?)))
        .collect()
}

fn value(given: &Json) -> Result<Value, String> {
    Ok(match given {
        Json::Null => return Err("TOML has no null: leave the key out".to_string()),
        Json::Bool(flag) => Value::Boolean(*flag),
        Json::Number(number) => match number.as_i64() {
            Some(whole) => Value::Integer(whole),
            None => Value::Float(number.as_f64().ok_or("a number TOML cannot hold")?),
        },
        Json::String(text) => Value::String(text.clone()),
        Json::Array(items) => Value::Array(items.iter().map(value).collect::<Result<_, _>>()?),
        Json::Object(inner) => Value::Table(table(inner)?),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str = "# a surface's own\n[Xmip]\nSurface = \"local\" # trails\n\
                            Clusters = [{ Surface = \"snapshot\", Snapshot = \"a.xmip\" }]\n\
                            [Logging.LogLevel]\nDefault = \"Information\"\n\
                            [numbers]\nwhole = 7\nhalf = 0.5\nwhen = 2026-11-17T10:00:00Z\n";

    #[test]
    fn a_document_reads_as_json_with_its_tables_arrays_and_values() {
        let read: Json = serde_json::from_str(&read(DOCUMENT).expect("TOML")).expect("JSON");

        assert_eq!(read["Xmip"]["Surface"], "local");
        assert_eq!(read["Xmip"]["Clusters"][0]["Snapshot"], "a.xmip");
        assert_eq!(read["Logging"]["LogLevel"]["Default"], "Information");
        assert_eq!(read["numbers"]["whole"], 7);
        assert_eq!(read["numbers"]["half"], 0.5);
        assert_eq!(read["numbers"]["when"], "2026-11-17T10:00:00Z");
    }

    #[test]
    fn text_that_is_not_toml_is_refused_in_the_readers_words() {
        let refused = read("[Xmip\nSurface =").expect_err("not TOML");

        assert!(refused.starts_with(configure::PARSE_FAILED), "{refused}");
    }

    #[test]
    fn json_written_as_toml_reads_back_as_the_same_json() {
        let json =
            r#"{"top":{"word":"said","count":2},"outer":{"inner":{"flag":true},"empty":{}}}"#;

        let written = write(json).expect("writes");

        assert!(written.contains("[outer.inner]"), "{written}");
        let back: Json = serde_json::from_str(&read(&written).expect("reads")).expect("JSON");
        assert_eq!(back, serde_json::from_str::<Json>(json).expect("JSON"));
    }

    #[test]
    fn a_null_or_a_document_that_is_no_object_is_refused() {
        assert!(write("[1]").is_err());
        assert!(
            write(r#"{"a":null}"#)
                .expect_err("null")
                .contains("no null")
        );
    }

    #[test]
    fn one_key_is_set_and_removed_and_the_rest_stays_as_written() {
        let set_to = set(
            DOCUMENT,
            r#"{"table":["Xmip"],"key":"Surface","value":"remote"}"#,
        )
        .expect("set");

        assert_eq!(set_to, DOCUMENT.replace("\"local\"", "\"remote\""));

        let removed = set(
            &set_to,
            r#"{"table":["numbers"],"key":"half","value":null}"#,
        )
        .expect("removed");
        assert!(!removed.contains("half"), "{removed}");
        assert!(removed.contains("# trails"), "{removed}");

        assert!(
            set(DOCUMENT, r#"{"table":["Xmip"]}"#).is_err(),
            "no key named"
        );
    }
}
