//! Topology of the poldercast overlay.
//!
//! In other words: how the nodes are connected to each other, how the will be
//! maintaining the links between them.
//!
//! The [`Topology`] object is maintaining the relative local topology of the
//! given Node.
//!
mod cyclon;
mod module;
mod ring;
mod vicinity;

pub use self::cyclon::Cyclon;
pub use self::module::Module;
pub use self::ring::Rings;
pub use self::vicinity::Vicinity;

use crate::{Id, Node};
use std::collections::BTreeMap;

pub struct Topology {
    our_node: Node,

    known_nodes: BTreeMap<Id, Node>,

    modules: BTreeMap<&'static str, Box<dyn Module + Send + Sync>>,
}

impl Topology {
    pub fn new(our_node: Node) -> Self {
        Topology {
            our_node: our_node,
            known_nodes: BTreeMap::new(),
            modules: BTreeMap::new(),
        }
    }

    /// create a new topology with the default poldercast's modules: [`Rings`],
    /// [`Vicinity`] and [`Cyclon`].
    ///
    pub fn default(our_node: Node) -> Self {
        let mut topology = Topology::new(our_node);
        topology.add_module(Rings::new());
        topology.add_module(Vicinity::new());
        topology.add_module(Cyclon::new());
        topology
    }

    /// add a module to participate into building the local topology (i.e.
    /// the link of nodes this module may connect to).
    ///
    /// It is recommended to use the default poldercast's modules: [`Rings`],
    /// [`Vicinity`] and [`Cyclon`]. Seed [`default`].
    ///
    #[inline]
    pub fn add_module<M: Module + Send + Sync + 'static>(&mut self, module: M) {
        let name = module.name();
        self.modules.insert(name, Box::new(module));
    }

    /// this is the view, the Nodes that the we need to contact in our neighborhood
    /// in order to propagate gossips (within other things).
    pub fn view(&self) -> Vec<Node> {
        let mut view = BTreeMap::new();

        for module in self.modules.values() {
            module.view(&self.known_nodes, &mut view)
        }

        view.into_iter().map(|v| v.1).collect()
    }

    /// update the known nodes and list of subscribers via the given collection
    /// of new node.
    ///
    /// This function can be called initially to bootstrap the topology with static
    /// values. But it is intended to be called at every gossips received from
    /// other nodes.
    pub fn update(&mut self, mut new_nodes: BTreeMap<Id, Node>) {

        new_nodes.remove(self.our_node.id());

        self.our_node.subscribers.extend(new_nodes.keys());
        self.known_nodes.extend(new_nodes);

        for module in self.modules.values_mut() {
            module.update(&self.our_node, &self.known_nodes);
        }
    }

    /// select the gossips to share with the given Node.
    ///
    /// This function requires the Topology object to be mutable because we will update
    /// timestamp regarding the last time we gossiped. This information can be useful
    /// for other nodes
    ///
    pub fn select_gossips(&mut self, gossip_recipient: &Node) -> BTreeMap<Id, Node> {
        let mut gossips = BTreeMap::new();

        self.our_node.last_gossip = std::time::SystemTime::now();

        for module in self.modules.values() {
            gossips.extend(module.select_gossips(
                &self.our_node,
                gossip_recipient,
                &self.known_nodes,
            ));
        }

        // Sanitize the gossip if the modules did not:
        // - the recipient does not need gossip about itself;
        // - we always include gossip about ourselves,
        //   with the updated timestamp.
        gossips.remove(gossip_recipient.id());
        gossips.insert(*self.our_node.id(), self.our_node.clone());

        gossips
    }
}
