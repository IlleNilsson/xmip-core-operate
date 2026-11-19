//! What a remote surface presents and trusts (ADR-0063 clause 1): the PEM
//! files a host's document names — its certificate chain and key, or a
//! chain whose key the platform's keeper holds and signs with (ADR-0034,
//! amendment 2026-10-15) — and the anchors a node's chain must reach, never
//! the operating system's trust store. Named together or not at all, each
//! there, checked as the document is read; what is missing is said when a
//! node is opened.

use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The files a remote surface presents and trusts, as named.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Named {
    pub certificate: Option<PathBuf>,
    pub key: Option<PathBuf>,
    pub anchors: Option<PathBuf>,
    /// Whether the platform's keeper holds the key: a certificate named with
    /// no key, where a host hands a keeper.
    pub kept: bool,
}

/// What a remote surface naming no anchors is told, its key in a file or
/// kept.
const NO_ANCHORS: &str = "REFUSED. A remote surface trusts a node by the anchors \
    Xmip:TrustAnchor or XMIP_TRUST_ANCHOR names, never the operating system's (ADR-0063)";

impl Named {
    /// The files named, checked: where a keeper `keeps` the key and a
    /// certificate is named with none, kept and read as a node is opened;
    /// otherwise named together or not at all, each one there.
    ///
    /// # Errors
    ///
    /// A certificate without a key, or a key without a certificate, where no
    /// keeper keeps it; a file named that is not there.
    pub fn checked(
        certificate: Option<PathBuf>,
        key: Option<PathBuf>,
        anchors: Option<PathBuf>,
        keeps: bool,
    ) -> Result<Self, String> {
        if keeps && certificate.is_some() && key.is_none() {
            return Ok(Self {
                certificate,
                key,
                anchors,
                kept: true,
            });
        }
        if certificate.is_some() != key.is_some() {
            return Err(
                "Xmip:Certificate and Xmip:PrivateKey are named together or not at all".into(),
            );
        }
        for named in [&certificate, &key, &anchors].into_iter().flatten() {
            if !named.is_file() {
                return Err(format!("{} is not there", named.display()));
            }
        }

        Ok(Self {
            certificate,
            key,
            anchors,
            kept: false,
        })
    }

    /// The identity a remote surface presents from files.
    ///
    /// # Errors
    ///
    /// No certificate and key named, no anchors named, or files that are no
    /// identity: the REFUSED sentence or the reader's words.
    pub fn identity(&self) -> Result<tls::Identity, String> {
        let (Some(certificate), Some(key)) = (&self.certificate, &self.key) else {
            return Err("REFUSED. A remote surface presents a certificate: name \
                Xmip:Certificate and Xmip:PrivateKey, or XMIP_CERTIFICATE and XMIP_PRIVATE_KEY \
                (ADR-0063 clause 1)"
                .to_string());
        };
        let Some(anchors) = &self.anchors else {
            return Err(NO_ANCHORS.to_string());
        };

        tls::Identity::from_pem(&read(certificate)?, &read(key)?, &read(anchors)?)
            .map_err(|failed| failed.message)
    }

    /// The identity a remote surface presents with the key `keeper`, called
    /// `called`, keeps: every signature asked of it.
    ///
    /// # Errors
    ///
    /// No certificate or anchors named or kept yet, or files that are no
    /// identity: the REFUSED sentence or the reader's words.
    pub fn kept_identity(
        &self,
        called: &str,
        keeper: Arc<dyn tls::Sign>,
    ) -> Result<tls::Identity, String> {
        let Some(certificate) = &self.certificate else {
            return Err(
                "REFUSED. No keeper holds this end's key; name Xmip:PrivateKey".to_string(),
            );
        };
        let Some(anchors) = &self.anchors else {
            return Err(NO_ANCHORS.to_string());
        };
        if !certificate.is_file() {
            return Err(format!(
                "REFUSED. No certificate is kept at {} yet: make a request over the key {called} \
                 holds and keep the chain the cluster's authority signs (ADR-0034)",
                certificate.display()
            ));
        }
        if !anchors.is_file() {
            return Err(format!(
                "REFUSED. No anchors are kept at {} yet: keep the cluster's authority, PEM, the \
                 one that signed the chain (ADR-0063)",
                anchors.display()
            ));
        }

        tls::Identity::from_signer(&read(certificate)?, keeper, &read(anchors)?)
            .map_err(|failed| failed.message)
    }
}

fn read(path: &Path) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|failed| failed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_named_together_and_there_unless_a_keeper_keeps_the_key() {
        let here = std::env::temp_dir().join("xmip-named-certificate.pem");
        std::fs::write(&here, "x").expect("written");
        let gone = std::env::temp_dir().join("xmip-named-nothing-here.pem");

        assert_eq!(
            Named::checked(Some(here.clone()), None, None, false)
                .err()
                .as_deref(),
            Some("Xmip:Certificate and Xmip:PrivateKey are named together or not at all")
        );
        assert!(Named::checked(Some(here.clone()), None, None, true).is_ok_and(|n| n.kept));
        assert_eq!(
            Named::checked(None, None, Some(gone.clone()), false).err(),
            Some(format!("{} is not there", gone.display()))
        );
        assert!(
            Named::default()
                .identity()
                .is_err_and(|said| said.contains("presents"))
        );
        let _ = std::fs::remove_file(here);
    }
}
