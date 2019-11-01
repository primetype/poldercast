use crate::{Id, NodeProfile, Nodes};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct GossipsBuilder {
    recipient: Id,

    gossips: HashSet<Id>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gossips(Vec<NodeProfile>);

impl GossipsBuilder {
    pub(crate) fn new(recipient: Id) -> Self {
        Self {
            recipient,
            gossips: HashSet::default(),
        }
    }

    pub fn recipient(&self) -> &Id {
        &self.recipient
    }

    pub fn add(&mut self, node: Id) -> bool {
        self.gossips.insert(node)
    }

    pub(crate) fn build(self, identity: NodeProfile, nodes: &Nodes) -> Gossips {
        let mut gossips = self
            .gossips
            .into_iter()
            .filter_map(|id| nodes.get(&id))
            .map(|node| node.profile().clone())
            .collect::<Vec<NodeProfile>>();
        gossips.push(identity);
        Gossips(gossips)
    }
}

impl Gossips {
    pub fn into_iter(self) -> impl Iterator<Item = NodeProfile> {
        self.0.into_iter()
    }

    pub fn inner(self) -> Vec<NodeProfile> {
        self.0
    }
}

impl From<Vec<NodeProfile>> for Gossips {
    fn from(gossips: Vec<NodeProfile>) -> Self {
        Gossips(gossips)
    }
}
