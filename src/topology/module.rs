use crate::{Id, Node};
use std::collections::BTreeMap;

pub trait Module {
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

    fn view<'a>(&self, known_nodes: &'a BTreeMap<Id, Node>) -> BTreeMap<Id, &'a Node>;
}
