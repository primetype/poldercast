use crate::{Id, Node};
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
        our_node: &Node,
        gossip_recipient: &Node,
        known_nodes: &BTreeMap<Id, Node>,
    ) -> BTreeMap<Id, Node>;

    /// update the Module's internal state based on the Node's subscribers
    /// and subscriptions.
    fn update(&mut self, our_node: &Node, known_nodes: &BTreeMap<Id, Node>);

    /// show the view of the module. This is the Node the module have
    /// selected as nodes of interest and/or relevant nodes to exchange
    /// gossips or any other messages.
    fn view<'a>(&self, known_nodes: &'a BTreeMap<Id, Node>) -> BTreeMap<Id, &'a Node>;
}
