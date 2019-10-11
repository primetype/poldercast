use crate::{Id, NodeData};
use std::collections::BTreeMap;

/// topology module. Defines a strategy to link nodes together.
///
pub trait Module {
    /// a unique identifier name of the given module.
    ///
    /// Useful for debugs and monitoring modules' activities.
    fn name(&self) -> &'static str;

    /// select gossips (node's Id to talk about) to send to another node.
    ///
    fn select_gossips(
        &self,
        our_node: &NodeData,
        gossip_recipient: &NodeData,
        known_nodes: &BTreeMap<Id, NodeData>,
    ) -> BTreeMap<Id, NodeData>;

    /// update the Module's internal state based on the NodeData's subscribers
    /// and subscriptions.
    fn update(&mut self, our_node: &NodeData, known_nodes: &BTreeMap<Id, NodeData>);

    /// show the view of the module. This is the NodeData the module have
    /// selected as nodes of interest and/or relevant nodes to exchange
    /// gossips or any other messages.
    fn view(&self, known_nodes: &BTreeMap<Id, NodeData>, view: &mut BTreeMap<Id, NodeData>);
}

/// filter module, will be applied on the set of modules to filter out undesired nodes
pub trait FilterModule {
    fn name(&self) -> &'static str;

    /// take ownership of some tree of nodes and filter out nodes that needs to be
    /// removed based on the filtering policies.
    fn filter(
        &self,
        _our_node: &NodeData,
        _other_nodes: BTreeMap<Id, NodeData>,
    ) -> BTreeMap<Id, NodeData>;
}
