use crate::{Address, Id, NodeProfile, NodeRef};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GossipsBuilder {
    recipient: NodeRef,

    gossips: HashMap<Id, NodeRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gossips(Vec<Gossip>);

/// message that is exchanged about a [`Node`] between gossiping nodes.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Gossip {
    pub(crate) node: NodeProfile,
}

impl GossipsBuilder {
    pub(crate) fn new(recipient: NodeRef) -> Self {
        Self {
            recipient,
            gossips: HashMap::default(),
        }
    }

    pub fn recipient(&self) -> &NodeRef {
        &self.recipient
    }

    pub fn add(&mut self, node: NodeRef) -> Option<NodeRef> {
        self.gossips.insert(node.public_id(), node)
    }

    pub(crate) fn build(self) -> Gossips {
        Gossips(
            self.gossips
                .into_iter()
                .map(|(_, node_ref)| Gossip::new(node_ref))
                .collect(),
        )
    }
}

impl Gossip {
    fn new(node_ref: NodeRef) -> Self {
        Gossip {
            node: node_ref.node().profile().clone(),
        }
    }

    pub(crate) fn public_id(&self) -> &Id {
        self.node.public_id()
    }

    pub(crate) fn address(&self) -> Option<&Address> {
        self.node.address()
    }
}

impl Gossips {
    pub(crate) fn into_iter(self) -> impl Iterator<Item = Gossip> {
        self.0.into_iter()
    }

    pub(crate) fn find(&self, public_id: &Id) -> Option<&NodeProfile> {
        self.0
            .iter()
            .map(|gossip| &gossip.node)
            .find(|profile| profile.public_id() == public_id)
    }
}
