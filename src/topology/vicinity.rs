/// The Vicinity module is responsible for maintaining interest-induced
/// random links, that is, randomly chosen links between nodes that share
/// one or more topics. Such links serve as input to the Rings module.
/// Additionally, they are used by the dissemination protocol to propagate
/// events to arbitrary subscribers of a topic.
use crate::{topology::Module, Id, Node};
use std::collections::{BTreeMap, BTreeSet};

pub const VICINITY_MAX_VIEW_SIZE: usize = 50;

pub const VICINITY_MAX_GOSSIP_LENGTH: usize = 32;

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
        _: &Node,
        gossip_recipient: &Node,
        known_nodes: &BTreeMap<Id, Node>,
    ) -> BTreeMap<Id, Node> {
        self.select_closest_nodes(gossip_recipient, known_nodes, VICINITY_MAX_GOSSIP_LENGTH)
    }

    /// update the Module's internal state based on the Node's subscribers
    /// and subscriptions.
    fn update(&mut self, our_node: &Node, known_nodes: &BTreeMap<Id, Node>) {
        self.view = self
            .select_closest_nodes(our_node, known_nodes, VICINITY_MAX_VIEW_SIZE)
            .keys()
            .cloned()
            .collect();
    }

    fn view<'a>(&self, known_nodes: &'a BTreeMap<Id, Node>) -> BTreeMap<Id, &'a Node> {
        let mut view = BTreeMap::new();
        for id in self.view.iter() {
            if let Some(node) = known_nodes.get(id) {
                view.insert(*id, node);
            } else {
                unreachable!()
            }
        }
        view
    }
}
impl Vicinity {
    /// create the new vicinity module with the given profile view
    ///
    /// at creating the vicinity module will initialize its own view
    /// using the given Profiles.
    pub fn new() -> Self {
        Vicinity {
            view: BTreeSet::new(),
        }
    }

    /// select nodes based on the proximity function (see Profile's proximity
    /// member function).
    fn select_closest_nodes(
        &self,
        to: &Node,
        known_nodes: &BTreeMap<Id, Node>,
        max: usize,
    ) -> BTreeMap<Id, Node> {
        let mut profiles: Vec<_> = known_nodes.values().collect();

        profiles.sort_by(|left, right| to.proximity(left).cmp(&to.proximity(right)));

        profiles
            .into_iter()
            .map(|v| (*v.id(), v.clone()))
            .take(max)
            .collect()
    }
}
