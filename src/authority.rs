//! The cluster's authority kept in a folder, and a node's certificate
//! carried across an air gap (ADR-0034, amendment 2026-10-22), for every
//! surface: an act and its arguments as JSON, the answer as JSON. Every act
//! is the capability's (`provision::register`, `provision::carried`,
//! `provision::renewal`); this reads the arguments and says the answer,
//! once — the Rust command line calls it, and `xmip_operate.h` section 23
//! answers with it.

use std::path::{Path, PathBuf};

use codec::pem;
use provision::Issuer;
use provision::carried;
use provision::crl::{self, Reason};
use provision::register::{Record, Register};
use provision::renewal::Standing;
use serde_json::{Value, json};
use tls::certificate::Certificate;

/// How long an authority set up here is valid: ten years. The assistant's
/// draft of 2026-10-22, for the owner to strike.
const AUTHORITY_LIFETIME: i64 = 10 * 365 * 86_400;

/// The act `act` on `arguments`, a JSON object, now.
///
/// # Errors
///
/// The act is none, its arguments are not what it takes, or the capability
/// refused it: the refusal in words.
pub fn act(act: &str, arguments: &Value) -> Result<Value, String> {
    acted(act, &Given(arguments), provision::now())
}

/// The arguments an act was given.
struct Given<'a>(&'a Value);

impl Given<'_> {
    fn text(&self, key: &str) -> Result<&str, String> {
        self.0
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| format!("the act needs {key}"))
    }

    fn optional(&self, key: &str) -> Option<&str> {
        self.text(key).ok()
    }

    fn path(&self, key: &str) -> Result<PathBuf, String> {
        self.text(key).map(PathBuf::from)
    }

    fn names(&self) -> Vec<String> {
        self.0
            .get("names")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect()
    }
}

/// One act, answered.
fn acted(act: &str, given: &Given<'_>, now: i64) -> Result<Value, String> {
    let said = |failure: provision::ProvisionError| failure.message;
    match act {
        "set-up" => set_up(given, now),
        "inspect" => inspect(&given.path("authority")?, now),
        "read" => read(&given.path("certificate")?, now),
        "revoke" => {
            let issuer = Issuer::kept(&given.path("authority")?).map_err(said)?;
            let reason =
                Reason::named(given.optional("reason").unwrap_or("unspecified")).map_err(said)?;
            let record = issuer
                .revoke(given.text("serial")?, reason, now)
                .map_err(said)?;
            Ok(record_json(&record))
        }
        "export" => export(given, now),
        "back-up" => {
            let register = Register::at(&given.path("authority")?);
            let copied = register.back_up(&given.path("folder")?).map_err(said)?;
            let copied: Vec<String> = copied.iter().map(|path| shown(path)).collect();
            Ok(json!({ "copied": copied }))
        }
        "request" => {
            let asked =
                carried::request_into(given.text("node")?, &given.names(), &given.path("folder")?)
                    .map_err(said)?;
            Ok(json!({
                "request": shown(&asked.request),
                "key": shown(&asked.key),
                "names": asked.names,
            }))
        }
        "sign" => sign(given, now),
        "import" => {
            let path = given.path("certificate")?;
            let text = std::fs::read_to_string(&path)
                .map_err(|failure| format!("{}: {failure}", path.display()))?;
            let imported = carried::import(&text, &given.path("folder")?, now).map_err(said)?;
            Ok(json!({
                "certificate": shown(&imported.certificate),
                "key": shown(&imported.key),
                "anchors": shown(&imported.anchors),
                "serial": imported.serial,
                "notAfter": imported.not_after,
                "renewAt": imported.renew_at,
                "anchorFingerprint": imported.anchor_fingerprint,
                "anchorTaken": imported.anchor_taken,
            }))
        }
        other => Err(format!(
            "{other:?} is no act on the cluster's authority; one of set-up, inspect, read, \
             revoke, export, back-up, request, sign, import"
        )),
    }
}

/// The authority kept in `authority`, made there called `name` where none
/// is, and the address it publishes at where `publish` says one.
fn set_up(given: &Given<'_>, now: i64) -> Result<Value, String> {
    let place = given.path("authority")?;
    let issuer = Issuer::kept_or_made(&place, given.text("name")?, now, AUTHORITY_LIFETIME)
        .map_err(|failure| failure.message)?;
    if let (Some(address), Some(register)) = (given.optional("publish"), issuer.register()) {
        register
            .publish(address)
            .map_err(|failure| failure.message)?;
    }
    inspect(&place, now)
}

/// The authority and everything it signed.
fn inspect(place: &Path, now: i64) -> Result<Value, String> {
    let said = |failure: provision::ProvisionError| failure.message;
    let issuer = Issuer::kept(place).map_err(said)?;
    let register = issuer.register().ok_or("the authority keeps no register")?;
    let own = Certificate::first(issuer.certificate()).map_err(|failure| failure.message)?;
    let anchor = pem::decode("CERTIFICATE", issuer.certificate())
        .map_err(|failure| failure.message)?
        .pop()
        .unwrap_or_default();
    let listed = crl::read(&issuer.revocation_list(now).map_err(said)?).map_err(said)?;
    let records: Vec<Value> = register
        .records()
        .map_err(said)?
        .iter()
        .map(|record| {
            let mut shown = record_json(record);
            shown["standing"] = json!(record.standing(now));
            shown
        })
        .collect();
    Ok(json!({
        "authority": shown(place),
        "subject": own.common_name().map_err(|failure| failure.message)?.unwrap_or_default(),
        "notBefore": own.not_before,
        "notAfter": own.not_after,
        "fingerprint": provision::fingerprint(&anchor),
        "published": register.published().map_err(said)?,
        "list": {
            "number": listed.number,
            "thisUpdate": listed.this_update,
            "nextUpdate": listed.next_update,
            "revoked": listed.revoked.len(),
        },
        "issued": records,
    }))
}

/// A certificate file's leaf and its standing: what a node's operator
/// reads to know when a request is due.
fn read(path: &Path, now: i64) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|failure| format!("{}: {failure}", path.display()))?;
    let leaf = Certificate::first(&text).map_err(|failure| failure.message)?;
    let standing = Standing::at(&leaf, now);
    Ok(json!({
        "certificate": shown(path),
        "serial": leaf.serial_hex(),
        "subject": leaf.common_name().map_err(|failure| failure.message)?.unwrap_or_default(),
        "names": leaf.names().map_err(|failure| failure.message)?,
        "notBefore": standing.not_before,
        "notAfter": standing.not_after,
        "renewAt": standing.renew_at,
        "standing": standing.word(),
    }))
}

/// The authority's certificate, PEM or DER, or its revocation list, DER,
/// written to `path`, never over a file.
fn export(given: &Given<'_>, now: i64) -> Result<Value, String> {
    let said = |failure: provision::ProvisionError| failure.message;
    let issuer = Issuer::kept(&given.path("authority")?).map_err(said)?;
    let path = given.path("path")?;
    let format = given
        .optional("format")
        .unwrap_or("pem")
        .to_ascii_lowercase();
    let bytes = match format.as_str() {
        "pem" => issuer.certificate().as_bytes().to_vec(),
        "der" => pem::decode("CERTIFICATE", issuer.certificate())
            .map_err(|failure| failure.message)?
            .pop()
            .unwrap_or_default(),
        "crl" => issuer.revocation_list(now).map_err(said)?,
        other => {
            return Err(format!(
                "{other:?} is no form the authority is exported in; pem, der or crl"
            ));
        }
    };
    secret::private_file::create_readable(&path, &bytes).map_err(|failure| {
        if failure.kind() == std::io::ErrorKind::AlreadyExists {
            format!(
                "{} is there already; nothing is overwritten.",
                path.display()
            )
        } else {
            format!("writing {}: {failure}", path.display())
        }
    })?;
    Ok(json!({ "written": shown(&path), "format": format }))
}

/// A node's carried request signed, the chain written where `out` says.
fn sign(given: &Given<'_>, now: i64) -> Result<Value, String> {
    let said = |failure: provision::ProvisionError| failure.message;
    let request = given.path("request")?;
    let text = std::fs::read_to_string(&request)
        .map_err(|failure| format!("{}: {failure}", request.display()))?;
    let signed =
        carried::sign(&given.path("authority")?, &text, &given.names(), now).map_err(said)?;
    let written = given.optional("out").map(PathBuf::from);
    if let Some(path) = &written {
        provision::operator::keep(path, &signed.chain, false).map_err(said)?;
    }
    Ok(json!({
        "request": shown(&request),
        "serial": signed.serial,
        "names": signed.names,
        "notAfter": signed.not_after,
        "written": written.as_deref().map(shown),
        "chain": signed.chain,
    }))
}

fn record_json(record: &Record) -> Value {
    json!({
        "serial": record.serial,
        "subject": record.subject,
        "names": record.names,
        "purpose": record.purpose,
        "notBefore": record.not_before,
        "notAfter": record.not_after,
        "renewAt": record.renew_at,
        "renewalId": record.renewal_id,
        "revoked": record.revoked.as_ref().map(|revoked| json!({
            "at": revoked.at,
            "reason": revoked.reason.word(),
        })),
    })
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_act_that_is_none_and_one_missing_its_arguments_are_refused_in_words() {
        let refused = act("rotate", &json!({})).expect_err("no such act");
        assert!(
            refused.contains("is no act on the cluster's authority"),
            "{refused}"
        );

        let refused = act("inspect", &json!({})).expect_err("no authority");
        assert_eq!(refused, "the act needs authority");
    }

    #[test]
    fn an_authority_set_up_is_inspected_with_what_it_signed() {
        let place = file::temporary::Folder::new("operate-authority");
        let authority = place.join("authority");
        let set_up = act(
            "set-up",
            &json!({ "authority": authority.display().to_string(), "name": "test authority" }),
        )
        .expect("set up");

        assert_eq!(set_up["subject"], "test authority");
        assert_eq!(set_up["issued"], json!([]));
        assert_eq!(set_up["list"]["revoked"], 0);
    }
}
