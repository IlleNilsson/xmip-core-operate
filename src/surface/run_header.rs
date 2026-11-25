//! What a run was started with, as its publisher says it under `[run]`
//! (`observe::Run`): the cluster, the tests, the nodes and the roles and
//! Node capabilities each declared, which are real, and how hard — said in
//! one line every surface puts at the top (the owner, 2026-09-19).

use observe::Run;

use crate::surface::node_capability::NodeCapability;

/// A run's header; [`RunHeader::default`] where the publisher says none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RunHeader {
    pub run: Run,
}

impl RunHeader {
    /// The header `run` says, or none.
    #[must_use]
    pub fn from(run: Option<&Run>) -> Self {
        Self {
            run: run.cloned().unwrap_or_default(),
        }
    }

    /// Whether the publisher said anything of a run.
    #[must_use]
    pub fn said(&self) -> bool {
        let run = &self.run;
        !run.cluster.is_empty()
            || !run.tests.is_empty()
            || !run.nodes.is_empty()
            || !run.stress.is_empty()
    }

    /// What the run started `node` with, its Node capabilities as its list
    /// says; [`NodeCapability::none`] where it names none.
    #[must_use]
    pub fn capability(&self, node: &str) -> NodeCapability {
        self.run
            .roles
            .iter()
            .map(|entry| NodeCapability::started(entry))
            .find(|started| started.node == node)
            .map_or_else(NodeCapability::none, |mut started| {
                started.capabilities = self
                    .run
                    .capabilities
                    .iter()
                    .map(|entry| node::Capability::from_declared_entry(entry))
                    .find(|(named, _)| *named == node)
                    .and_then(|(_, words)| words.ok())
                    .unwrap_or_default();
                started
            })
    }

    /// The one line: tests, cluster, nodes as each declared, their Node
    /// capabilities, real and stress, joined by ` · `; empty where
    /// nothing was said.
    #[must_use]
    pub fn line(&self) -> String {
        if !self.said() {
            return String::new();
        }

        let run = &self.run;
        let mut parts = Vec::new();

        if !run.tests.is_empty() {
            parts.push(run.tests.join(" "));
        }
        if !run.cluster.is_empty() {
            parts.push(run.cluster.clone());
        }
        if run.nodes.is_empty() {
            parts.push("no nodes".to_string());
        } else {
            let declared: Vec<&str> = run.nodes.iter().map(|node| self.declared(node)).collect();
            parts.push(format!("nodes {}", declared.join(" ")));
            parts.push(if run.capabilities.is_empty() {
                "no Node capability".to_string()
            } else {
                format!("Node capabilities {}", run.capabilities.join(" "))
            });
            if !run.real.is_empty() {
                parts.push(format!("real {}, the rest simulated", run.real.join(" ")));
            }
        }
        if !run.stress.is_empty() {
            parts.push(run.stress.clone());
        }

        parts.join(" · ")
    }

    // A node as its run's roles entry declares it, or its bare name.
    fn declared<'a>(&'a self, node: &'a str) -> &'a str {
        self.run
            .roles
            .iter()
            .find(|entry| NodeCapability::started(entry).node == node)
            .map_or(node, String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_run_is_one_line_and_none_is_empty() {
        let test = configure::fixture::test_cluster();
        let (a, b) = (&test.node(0).name, &test.node(1).name);
        let declared = format!("{a}={}", test.node(0).roles[0]);
        let run = Run {
            cluster: test.name.clone(),
            tests: vec!["t".into()],
            nodes: vec![a.clone(), b.clone()],
            roles: vec![declared.clone()],
            capabilities: vec![format!("{a}=online+erp-direct")],
            stress: "light".into(),
            ..Run::default()
        };
        let header = RunHeader::from(Some(&run));

        assert_eq!(
            header.line(),
            format!(
                "t · {} · nodes {declared} {b} · Node capabilities {a}=online+erp-direct · light",
                test.name
            )
        );
        assert_eq!(header.capability(a).capabilities, ["online", "erp-direct"]);
        assert!(!header.capability(b).said());
        assert_eq!(RunHeader::default().line(), "");
    }
}
