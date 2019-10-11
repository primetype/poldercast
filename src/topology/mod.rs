//! Topology of the poldercast overlay.
//!
//! In other words: how the nodes are connected to each other, how the will be
//! maintaining the links between them.
//!
//! The [`Topology`] object is maintaining the relative local topology of the
//! given NodeData.
//!
mod cyclon;
mod module;
mod ring;
mod vicinity;

pub use self::cyclon::Cyclon;
pub use self::module::{FilterModule, Module};
pub use self::ring::Rings;
pub use self::vicinity::Vicinity;

use crate::{Id, Node, NodeData};
use std::collections::BTreeMap;

/// Topology manager
///
/// will provide the information for what nodes to talk to and what
/// nodes we will need to share gossips to.
///
/// It is possible to customize the different modules of poldercast
/// by adding different modules but the default ones.
///
/// Another thing that can be done is filter before hands some nodes that
/// are not desirable by setting the `gossip_filter`.
pub struct Topology {
    our_node: Node,

    known_nodes: BTreeMap<Id, NodeData>,

    modules: BTreeMap<&'static str, Box<dyn Module + Send + Sync>>,

    filter_modules: BTreeMap<&'static str, Box<dyn FilterModule + Send + Sync>>,
}

impl Topology {
    pub fn new(our_node: Node) -> Self {
        let mut topology = Topology {
            our_node,
            known_nodes: BTreeMap::new(),
            modules: BTreeMap::new(),
            filter_modules: BTreeMap::new(),
        };

        topology.add_filter_module(DefaultFilterModule::default());

        topology
    }

    /// create a new topology with the default poldercast's modules: [`Rings`],
    /// [`Vicinity`] and [`Cyclon`].
    ///
    pub fn default(our_node: Node) -> Self {
        let mut topology = Topology::new(our_node);
        topology.add_module(Rings::default());
        topology.add_module(Vicinity::default());
        topology.add_module(Cyclon::default());
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

    /// add a filter module that will participate in the policy to pre-filter nodes
    /// received from the gossiping.
    ///
    /// There is no need to filter NodeDatas with no addresses as they are already filtered
    /// by default
    #[inline]
    pub fn add_filter_module<FM: FilterModule + Send + Sync + 'static>(
        &mut self,
        filter_module: FM,
    ) {
        let name = filter_module.name();
        self.filter_modules.insert(name, Box::new(filter_module));
    }

    /// this is the view, the NodeDatas that the we need to contact in our neighborhood
    /// in order to propagate gossips (within other things).
    pub fn view(&self) -> Vec<NodeData> {
        let mut view = BTreeMap::new();

        for module in self.modules.values() {
            module.view(&self.known_nodes, &mut view)
        }

        view.into_iter().map(|v| v.1).collect()
    }

    fn filter_nodes(&self, mut nodes: BTreeMap<Id, NodeData>) -> BTreeMap<Id, NodeData> {
        for filter in self.filter_modules.values() {
            nodes = filter.filter(&self.our_node.data(), nodes);
        }

        nodes
    }

    /// update the known nodes and list of subscribers via the given collection
    /// of new node.
    ///
    /// This function can be called initially to bootstrap the topology with initial
    /// values. But it is intended to be called at every gossips received from
    /// other nodes.
    ///
    /// this function will be filtering NodeDatas that do not have IP public address
    /// (i.e. `node.address().is_some()`).
    pub fn update(&mut self, new_nodes: BTreeMap<Id, NodeData>) {
        let filtered_nodes = self.filter_nodes(new_nodes);

        self.our_node
            .data_mut()
            .subscribers
            .extend(filtered_nodes.keys());
        self.known_nodes.extend(filtered_nodes);

        for module in self.modules.values_mut() {
            module.update(self.our_node.data(), &self.known_nodes);
        }
    }

    /// evict a node from the list of known nodes and returns it
    pub fn evict_node(&mut self, id: Id) -> Option<NodeData> {
        if let Some(node) = self.known_nodes.remove(&id) {
            let known_nodes = std::mem::replace(&mut self.known_nodes, BTreeMap::new());
            self.update(known_nodes);
            Some(node)
        } else {
            None
        }
    }

    /// select the gossips to share with the given NodeData.
    ///
    /// This function requires the Topology object to be mutable because we will update
    /// timestamp regarding the last time we gossiped. This information can be useful
    /// for other nodes
    ///
    pub fn select_gossips(&mut self, gossip_recipient: &NodeData) -> BTreeMap<Id, NodeData> {
        let mut gossips = BTreeMap::new();

        self.our_node.data_mut().last_gossip = std::time::SystemTime::now();

        let known_nodes = self
            .known_nodes
            .iter()
            .filter(|node| node.1.address().is_some())
            .map(|(id, node)| (*id, node.clone()))
            .collect();

        for module in self.modules.values() {
            gossips.extend(module.select_gossips(
                self.our_node.data(),
                gossip_recipient,
                &known_nodes,
            ));
        }

        // Sanitize the gossip if the modules did not:
        // - the recipient does not need gossip about itself;
        gossips.remove(gossip_recipient.id());

        gossips.insert(*self.our_node.data().id(), self.our_node.data().clone());

        gossips
    }
}

struct DefaultFilterModule;
impl Default for DefaultFilterModule {
    fn default() -> Self {
        DefaultFilterModule
    }
}
impl FilterModule for DefaultFilterModule {
    fn name(&self) -> &'static str {
        "default filter module"
    }

    fn filter(
        &self,
        our_node: &NodeData,
        other_nodes: BTreeMap<Id, NodeData>,
    ) -> BTreeMap<Id, NodeData> {
        other_nodes
            .into_iter()
            .filter(|(_id, node)| our_node.id() != node.id() && node.address().is_some())
            .collect()
    }
}
