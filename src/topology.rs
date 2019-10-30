use crate::nodes::Entry;
use crate::{
    DefaultPolicy, Gossips, GossipsBuilder, Id, Layer, NodeProfile, NodeRef, Nodes, Policy,
    Selection, ViewBuilder,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GossipingError {
    #[error("The node ({id:?}) does not advertise itself in the gossips")]
    NodeDoesNotAdvertiseSelf { id: Id },

    /// invalid gossip, the gossiping node has been attached to the error so
    /// it is possible to check if as a consequence the node has been quarantined
    ///
    #[error("The node has gossiped invalid gossip(s)")]
    InvalidGossip { node: NodeRef },
}

pub struct Topology {
    /// The local node identity
    profile: NodeProfile,

    nodes: Nodes,

    layers: Vec<Box<dyn Layer>>,

    policy: Box<dyn Policy>,
}

impl Topology {
    pub fn new(profile: NodeProfile) -> Self {
        Self {
            profile,
            nodes: Nodes::default(),
            layers: Vec::default(),
            policy: Box::new(DefaultPolicy::default()),
        }
    }

    pub fn profile(&self) -> &NodeProfile {
        &self.profile
    }

    pub fn view(&mut self, selection: Selection) -> Vec<NodeRef> {
        let mut view_builder = ViewBuilder::new(selection);

        for layer in self.layers.iter_mut() {
            layer.view(&mut view_builder)
        }

        view_builder.build()
    }

    fn find_node(&mut self, public_id: Id, gossips: &Gossips) -> Option<NodeRef> {
        match self.nodes.entry(public_id) {
            Entry::Occupied(entry) => Some(entry.release_mut().clone()),
            _ => gossips.find(&public_id).cloned().map(NodeRef::new),
        }
    }

    fn update_known_nodes(
        &mut self,
        _from: NodeRef,
        gossips: Gossips,
    ) -> Result<(), GossipingError> {
        for gossip in gossips.into_iter() {
            if gossip.public_id() == self.profile.public_id() {
                // ignore ourselves
                continue;
            }

            if let (Some(my_address), Some(other_address)) =
                (self.profile().address(), gossip.address())
            {
                if my_address == other_address {
                    // address theft or we have a new Id since then
                    continue;
                }
            }

            match self.nodes.entry(*gossip.public_id()) {
                Entry::Occupied(mut occupied) => {
                    occupied.modify(&mut self.policy, |node| node.update_gossip(gossip))
                }
                Entry::Vacant(mut vacant) => {
                    vacant.insert(NodeRef::new(gossip.node));
                }
            }
        }

        Ok(())
    }

    pub fn initiate_gossips(&mut self, with: NodeRef) -> Gossips {
        with.node_mut().logs_mut().gossiping();
        let mut gossips_builder = GossipsBuilder::new(with);

        for layer in self.layers.iter_mut() {
            layer.gossips(&self.profile, &mut gossips_builder, &self.nodes)
        }

        gossips_builder.build()
    }

    pub fn accept_gossips(&mut self, from: Id, gossips: Gossips) -> Result<(), GossipingError> {
        let from = if let Some(from) = self.find_node(from, &gossips) {
            from
        } else {
            return Err(GossipingError::NodeDoesNotAdvertiseSelf { id: from });
        };

        from.node_mut().logs_mut().gossiping();
        self.update_known_nodes(from, gossips)?;

        for layer in self.layers.iter_mut() {
            layer.reset();
            layer.populate(&self.profile, &self.nodes);
        }

        Ok(())
    }

    pub fn exchange_gossips(
        &mut self,
        with: &Id,
        gossips: Gossips,
    ) -> Result<Gossips, GossipingError> {
        let with = if let Some(with) = self.find_node(*with, &gossips) {
            with
        } else {
            return Err(GossipingError::NodeDoesNotAdvertiseSelf { id: *with });
        };

        with.node_mut().logs_mut().gossiping();
        self.update_known_nodes(with.clone(), gossips)?;

        let mut gossips_builder = GossipsBuilder::new(with);

        for layer in self.layers.iter_mut() {
            layer.reset();
            layer.populate(&self.profile, &self.nodes);
            layer.gossips(&self.profile, &mut gossips_builder, &self.nodes)
        }

        Ok(gossips_builder.build())
    }
}
