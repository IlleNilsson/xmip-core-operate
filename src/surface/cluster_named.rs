//! How every surface shows a cluster where it shows its name (ADR-0028,
//! amendment 2026-11-28): the short name its file declares, where the
//! publisher says one, else the name its run gives it; and the tooltip
//! beside it, which carries the file's name, the short name, the run's name
//! and the file (the owner, 2026-11-28: *Also all UI need to use tooltips
//! for explanation and underlying information*). Said here once: the
//! command line, the PowerShell module and every GUI ask this rule, Rust to
//! Rust or through `xmip_cluster_said_v1`.

use std::path::Path;

use observe::Naming;

use crate::surface::OperatorSurface;

/// A cluster as a surface names it: the name its run gives it and how its
/// publisher names it for a person.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClusterNamed {
    /// The cluster's name, as its run gives it: `-Cluster`, or a machine's
    /// `--cluster`.
    pub name: String,
    pub naming: Naming,
}

impl ClusterNamed {
    /// The cluster `surface` reads: the name its run says, else the one
    /// first segment beneath the root every published scope shares, else
    /// none; and how its publisher names it.
    #[must_use]
    pub fn of(surface: &dyn OperatorSurface) -> Self {
        let run = surface.run().run.cluster;
        let name = if run.is_empty() {
            let roots = surface.index().nodes();
            if roots.len() == 1 {
                roots[0].clone()
            } else {
                String::new()
            }
        } else {
            run
        };
        Self {
            name,
            naming: surface.naming(),
        }
    }

    /// What a surface shows for the cluster: its short name, else its name.
    #[must_use]
    pub fn shown(&self) -> &str {
        if self.naming.short_name.is_empty() {
            &self.name
        } else {
            &self.naming.short_name
        }
    }

    /// The tooltip beside it, each part the publisher said, joined by ` · `:
    /// the file's name, which is the cluster's full name, the short name,
    /// the name the run gives it and the file. Empty where nothing is said.
    #[must_use]
    pub fn title(&self) -> String {
        let file = self.naming.file.as_str();
        let full = Path::new(file)
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut parts = Vec::new();
        if !full.is_empty() {
            parts.push(full);
        }
        if !self.naming.short_name.is_empty() {
            parts.push(format!("short name {}", self.naming.short_name));
        }
        if !self.name.is_empty() {
            parts.push(format!("cluster {}", self.name));
        }
        if !file.is_empty() {
            parts.push(format!("from {file}"));
        }
        parts.join(" · ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(name: &str, short_name: &str, file: &str) -> ClusterNamed {
        ClusterNamed {
            name: name.to_string(),
            naming: Naming {
                short_name: short_name.to_string(),
                file: file.to_string(),
            },
        }
    }

    #[test]
    fn the_short_name_shows_and_the_tooltip_carries_the_rest() {
        let cluster = configure::fixture::test_cluster();
        let whole = named(&cluster.name, "Ab", "test/x/a-file.xmip");
        assert_eq!(whole.shown(), "Ab");
        assert_eq!(
            whole.title(),
            format!(
                "a-file · short name Ab · cluster {} · from test/x/a-file.xmip",
                cluster.name
            )
        );

        let node = named(&cluster.name, "Ab", "");
        assert_eq!(
            node.title(),
            format!("short name Ab · cluster {}", cluster.name)
        );
        let bare = named(&cluster.name, "", "");
        assert_eq!(bare.shown(), cluster.name);
        assert_eq!(bare.title(), format!("cluster {}", cluster.name));
        assert_eq!(ClusterNamed::default().title(), "");
    }
}
