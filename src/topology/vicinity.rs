/// The Vicinity module is responsible for maintaining interest-induced
/// random links, that is, randomly chosen links between nodes that share
/// one or more topics. Such links serve as input to the Rings module.
/// Additionally, they are used by the dissemination protocol to propagate
/// events to arbitrary subscribers of a topic.
use crate::{topology::Module, Id, NodeData};
use std::collections::{BTreeMap, BTreeSet};

pub const VICINITY_MAX_VIEW_SIZE: usize = 10;

pub const VICINITY_MAX_GOSSIP_LENGTH: usize = 20;

#[derive(Clone, Debug)]
pub struct Vicinity {
    pub(crate) view: BTreeSet<Id>,
}
impl Module for Vicinity {
    fn name(&self) -> &'static str {
        "vicinity"
    }

    /// select gossips (node's Id to talk about) to send to another node.
    ///
    fn select_gossips(
        &self,
        _: &NodeData,
        gossip_recipient: &NodeData,
        known_nodes: &BTreeMap<Id, NodeData>,
    ) -> BTreeMap<Id, NodeData> {
        self.select_closest_nodes(gossip_recipient, known_nodes, VICINITY_MAX_GOSSIP_LENGTH)
    }

    /// update the Module's internal state based on the NodeData's subscribers
    /// and subscriptions.
    fn update(&mut self, our_node: &NodeData, known_nodes: &BTreeMap<Id, NodeData>) {
        self.view = self
            .select_closest_nodes(our_node, known_nodes, VICINITY_MAX_VIEW_SIZE)
            .keys()
            .cloned()
            .collect();
    }

    fn view(&self, known_nodes: &BTreeMap<Id, NodeData>, view: &mut BTreeMap<Id, NodeData>) {
        for id in self.view.iter() {
            if let Some(node) = known_nodes.get(id) {
                view.insert(*id, node.clone());
            } else {
                unreachable!()
            }
        }
    }
}
impl Vicinity {
    /// select nodes based on the proximity function (see Profile's proximity
    /// member function).
    fn select_closest_nodes(
        &self,
        to: &NodeData,
        known_nodes: &BTreeMap<Id, NodeData>,
        max: usize,
    ) -> BTreeMap<Id, NodeData> {
        let mut profiles: Vec<_> = known_nodes.values().collect();

        profiles.sort_by(|left, right| to.proximity(left).cmp(&to.proximity(right)));

        profiles
            .into_iter()
            .map(|v| (*v.id(), v.clone()))
            .take(max)
            .collect()
    }
}

impl Default for Vicinity {
    fn default() -> Self {
        Vicinity {
            view: BTreeSet::default(),
        }
    }
}
