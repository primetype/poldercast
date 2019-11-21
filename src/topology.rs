use crate::nodes::Entry;
use crate::{
    DefaultPolicy, Gossips, GossipsBuilder, Id, Layer, Node, NodeInfo, NodeProfile, Nodes, Policy,
    PolicyReport, Selection, ViewBuilder,
};

pub struct Topology {
    /// The local node identity
    profile: NodeProfile,

    nodes: Nodes,

    layers: Vec<Box<dyn Layer + Send + Sync>>,

    policy: Box<dyn Policy + Send + Sync>,
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

    pub fn add_layer<L>(&mut self, layer: L)
    where
        L: Layer + Send + Sync + 'static,
    {
        self.layers.push(Box::new(layer));
    }

    pub fn set_policy<P>(&mut self, policy: P)
    where
        P: Policy + Send + Sync + 'static,
    {
        self.policy = Box::new(policy);
    }

    pub fn view(&mut self, from: Option<Id>, selection: Selection) -> Vec<NodeInfo> {
        let mut view_builder = ViewBuilder::new(selection);

        if let Some(from) = from {
            view_builder.with_origin(from);
        }

        for layer in self.layers.iter_mut() {
            layer.view(&mut view_builder, &mut self.nodes)
        }

        view_builder.build(&self.nodes)
    }

    fn update_known_nodes(&mut self, _from: Id, gossips: Gossips) {
        for gossip in gossips.into_iter() {
            if gossip.id() == self.profile.id() {
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

            match self.nodes.entry(*gossip.id()) {
                Entry::Occupied(mut occupied) => {
                    occupied.modify(&mut self.policy, |node| node.update_gossip(gossip));
                }
                Entry::Vacant(mut vacant) => {
                    vacant.insert(Node::new(gossip));
                }
            }
        }
    }

    pub fn initiate_gossips(&mut self, with: Id) -> Gossips {
        if let Some(with) = self.nodes.get_mut(&with) {
            with.logs_mut().gossiping();
        }
        let mut gossips_builder = GossipsBuilder::new(with);

        for layer in self.layers.iter_mut() {
            layer.gossips(&self.profile, &mut gossips_builder, &self.nodes)
        }

        gossips_builder.build(self.profile.clone(), &self.nodes)
    }

    /// reset the layers, allowing an update of the internal state
    ///
    pub fn force_reset_layers(&mut self) {
        self.reset_layers()
    }

    fn reset_layers(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.reset();
            layer.populate(&self.profile, &self.nodes);
        }
    }

    pub fn accept_gossips(&mut self, from: Id, gossips: Gossips) {
        if let Some(from) = self.nodes.get_mut(&from) {
            from.logs_mut().gossiping();
        }

        self.update_known_nodes(from, gossips);

        self.reset_layers();
    }

    pub fn exchange_gossips(&mut self, with: Id, gossips: Gossips) -> Gossips {
        if let Some(with) = self.nodes.get_mut(&with) {
            with.logs_mut().gossiping();
        }

        self.update_known_nodes(with, gossips);

        let mut gossips_builder = GossipsBuilder::new(with);

        for layer in self.layers.iter_mut() {
            layer.reset();
            layer.populate(&self.profile, &self.nodes);
            layer.gossips(&self.profile, &mut gossips_builder, &self.nodes)
        }

        gossips_builder.build(self.profile.clone(), &self.nodes)
    }

    pub fn update_node<F>(&mut self, id: Id, update: F) -> Option<PolicyReport>
    where
        F: FnOnce(&mut Node),
    {
        self.nodes.entry(id).and_modify(&mut self.policy, update)
    }
}
