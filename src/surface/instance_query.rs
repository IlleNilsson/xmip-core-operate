//! What a surface asks of the Work Processes' Instances it lists
//! (`runtime-model.md` section 22; ADR-0066, amendment 2026-11-25), in the
//! words every surface uses: where, `--location` — a cluster or a node, and
//! every Work Process run at or beneath it; which Work Process, by its
//! configured name; and one Instance, by its identifier. Written once:
//! `xmip instances` asks it here, and every .NET surface through section
//! 28's list query (`XMIP_LIST_INSTANCES`). A list is in the order its
//! nodes published it, each Work Process's Instances newest written first.

use observe::WorkProcessInstances;

use crate::surface::list_order::placed;

/// The one order a list of Instances takes: newest written first.
pub const COLUMNS: [&str; 1] = ["updated"];

/// What a line asks of the Work Processes' Instances.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstanceQuery {
    pub location: Option<String>,
    pub work_process: Option<String>,
    pub instance: Option<String>,
}

impl InstanceQuery {
    /// The Work Processes of `lists` the query selects, each with only the
    /// Instance it names where it names one; one left with none of it is
    /// left out.
    #[must_use]
    pub fn apply(&self, lists: &[WorkProcessInstances]) -> Vec<WorkProcessInstances> {
        self.places(lists)
            .into_iter()
            .map(|(at, instances)| WorkProcessInstances {
                instances: instances
                    .into_iter()
                    .map(|one| lists[at].instances[one].clone())
                    .collect(),
                ..lists[at].clone()
            })
            .collect()
    }

    /// The place in `lists` of each Work Process the query selects, and the
    /// places of its Instances it selects.
    #[must_use]
    pub fn places(&self, lists: &[WorkProcessInstances]) -> Vec<(usize, Vec<usize>)> {
        let named = |word: &Option<String>| word.clone().filter(|word| !word.trim().is_empty());
        let (process, instance) = (named(&self.work_process), named(&self.instance));
        lists
            .iter()
            .enumerate()
            .filter(|(_, list)| placed(&list.node, self.location.as_deref()))
            .filter(|(_, list)| {
                process
                    .as_ref()
                    .is_none_or(|name| list.work_process == *name)
            })
            .filter_map(|(at, list)| {
                let instances: Vec<usize> = list
                    .instances
                    .iter()
                    .enumerate()
                    .filter(|(_, one)| instance.as_ref().is_none_or(|id| one.instance == *id))
                    .map(|(place, _)| place)
                    .collect();
                (instance.is_none() || !instances.is_empty()).then_some((at, instances))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use configure::fixture::test_cluster;
    use observe::WorkProcessInstance;

    use super::*;

    fn list(node: &str, work_process: &str, ids: &[&str]) -> WorkProcessInstances {
        WorkProcessInstances {
            node: node.to_string(),
            work_process: work_process.to_string(),
            instances: ids
                .iter()
                .map(|id| WorkProcessInstance {
                    instance: (*id).to_string(),
                    ..WorkProcessInstance::default()
                })
                .collect(),
            ..WorkProcessInstances::default()
        }
    }

    #[test]
    fn the_query_chooses_where_the_work_process_and_one_instance() {
        let cluster = test_cluster();
        let (first, second) = (cluster.node_scope(0), cluster.node_scope(1));
        let lists = [
            list(&first, "Order in parts", &["a", "b"]),
            list(&first, "Order approval", &["c"]),
            list(&second, "Order in parts", &["d"]),
        ];
        let every = InstanceQuery::default();
        assert_eq!(every.places(&lists).len(), 3);
        let here = InstanceQuery {
            location: Some(first.clone()),
            ..InstanceQuery::default()
        };
        assert_eq!(here.places(&lists), [(0, vec![0, 1]), (1, vec![0])]);
        let named = InstanceQuery {
            work_process: Some("Order in parts".into()),
            ..InstanceQuery::default()
        };
        assert_eq!(named.places(&lists), [(0, vec![0, 1]), (2, vec![0])]);
        let one = InstanceQuery {
            instance: Some("b".into()),
            ..InstanceQuery::default()
        };
        let chosen = one.apply(&lists);
        assert_eq!(chosen.len(), 1, "only the Work Process holding it");
        assert_eq!(chosen[0].instances[0].instance, "b");
        let cluster_wide = InstanceQuery {
            location: Some(cluster.scope()),
            ..InstanceQuery::default()
        };
        assert_eq!(cluster_wide.apply(&lists).len(), 3);
    }
}
