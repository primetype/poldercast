use crate::{Address, NodeProfile, Nodes};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct GossipsBuilder {
    recipient: Address,

    gossips: HashSet<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gossips(Vec<NodeProfile>);

impl GossipsBuilder {
    pub(crate) fn new(recipient: Address) -> Self {
        Self {
            recipient,
            gossips: HashSet::default(),
        }
    }

    pub fn recipient(&self) -> &Address {
        &self.recipient
    }

    pub fn add(&mut self, node: Address) -> bool {
        self.gossips.insert(node)
    }

    pub(crate) fn build(self, identity: NodeProfile, nodes: &Nodes) -> Gossips {
        let mut gossips = self
            .gossips
            .into_iter()
            .filter_map(|id| nodes.peek(&id))
            .map(|node| node.profile().clone())
            .collect::<Vec<NodeProfile>>();
        gossips.push(identity);
        Gossips(gossips)
    }
}

impl Gossips {
    pub fn inner(self) -> Vec<NodeProfile> {
        self.0
    }
}

impl IntoIterator for Gossips {
    type Item = NodeProfile;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<Vec<NodeProfile>> for Gossips {
    fn from(gossips: Vec<NodeProfile>) -> Self {
        Gossips(gossips)
    }
}
