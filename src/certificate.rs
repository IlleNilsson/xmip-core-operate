//! A person's and a node's certificate, for every surface (ADR-0034,
//! amendment 2026-10-15; ADR-0015, amendment 2026-10-21): what is composed
//! of the capability's acts and said once — the name a cluster's authority
//! is made under, and a person's request signed by the authority a node's
//! configuration names, its chain kept whole where a file is named and
//! never over one. The making and the signing are `provision::operator`'s
//! and `runtime::operator_certificate`'s.

use std::path::Path;

/// The name the cluster `cluster`'s own authority is made under.
#[must_use]
pub fn authority_name(cluster: &str) -> String {
    format!("Xmip cluster {cluster} authority")
}

/// The request at `request` signed by the authority the node
/// configuration at `configuration` names in `[provision.operator]`, and
/// its chain kept whole at `chain` where it is named: the chain, PEM, leaf
/// first.
///
/// # Errors
///
/// No request at its path; a chain named that is there already; or the
/// authority refused, in its words opening `REFUSED:`.
pub fn sign_request(
    request: &Path,
    configuration: &Path,
    chain: Option<&Path>,
) -> Result<String, String> {
    if !request.is_file() {
        return Err(format!("No file at {}.", request.display()));
    }
    if let Some(chain) = chain.filter(|chain| chain.exists()) {
        return Err(format!(
            "REFUSED: {} is there already; nothing is overwritten.",
            chain.display()
        ));
    }

    let asked =
        std::fs::read_to_string(request).map_err(|failure| format!("REFUSED: {failure}"))?;
    let signed = runtime::operator_certificate::sign(configuration, &asked)
        .map_err(|refused| format!("REFUSED: {refused}"))?;

    if let Some(chain) = chain {
        provision::operator::keep(chain, &signed, false)
            .map_err(|refused| format!("REFUSED: {}", refused.message))?;
    }

    Ok(signed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_request_that_is_not_there_is_said_so_and_the_authority_is_named_for_its_cluster() {
        let folder = file::temporary::Folder::new("operate-no-request");
        let absent = folder.join("no-such-request.csr");

        assert_eq!(
            sign_request(&absent, Path::new("node.xmip"), None),
            Err(format!("No file at {}.", absent.display()))
        );
        assert_eq!(authority_name("c"), "Xmip cluster c authority");
    }
}
