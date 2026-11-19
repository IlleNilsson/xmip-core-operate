//! The secrets a node keeps, listed and set through its operate listener,
//! never a value read back (ADR-0019, amendment 2026-10-29), as this side
//! or as a caller this process holds.

use runtime::plane::remote::Outcome;

use super::ListenerSurface;
use crate::surface::OperatorSurface;
use crate::surface::acted::{SecretOperation, SecretRecord};

impl ListenerSurface {
    /// The secrets the node at `node` keeps, asked as `caller` (0: this
    /// side).
    ///
    /// # Errors
    ///
    /// Why they were not listed: the surface's source where no node is
    /// reached, else the node's words.
    pub fn secrets_as(&self, caller: u64, node: &str) -> Result<Vec<SecretRecord>, String> {
        let Some(remote) = &self.node else {
            return Err(self.source());
        };

        remote
            .secrets(caller, node)
            .map(|listed| SecretRecord::listed(&listed))
    }

    /// Set the secret `name` of `kind` to `value` on the node at `node`,
    /// asked as `caller` (0: this side). A node that did not answer is lost.
    pub fn set_secret_as(
        &self,
        caller: u64,
        node: &str,
        name: &str,
        (kind, value): (&str, &str),
    ) -> SecretOperation {
        let declined = |result: String| SecretOperation {
            node: node.to_string(),
            name: name.to_string(),
            applied: false,
            result,
        };
        let Some(remote) = &self.node else {
            return declined(self.source());
        };

        match remote.set_secret(caller, node, (name, kind, value)) {
            Ok((outcome, said)) => SecretOperation {
                applied: outcome == Outcome::Taken,
                ..declined(said)
            },
            Err(said) => {
                self.lost(said.clone());
                declined(said)
            }
        }
    }
}
