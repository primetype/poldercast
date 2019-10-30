use crate::{Id, NodeRef, Policy, PolicyReport};
use std::collections::{BTreeMap, HashMap};

#[derive(Default, Debug)]
pub struct Nodes {
    all: HashMap<Id, NodeRef>,

    quarantined: HashMap<Id, NodeRef>,

    available: BTreeMap<Id, NodeRef>,
}

pub enum Entry<'a> {
    Vacant(NodeEntry<'a>),
    Occupied(NodeEntry<'a>),
}

pub struct NodeEntry<'a> {
    nodes: &'a mut Nodes,
    public_id: Id,
}

impl Nodes {
    pub fn entry(&mut self, public_id: Id) -> Entry<'_> {
        if self.all.contains_key(&public_id) {
            Entry::Occupied(NodeEntry::new(self, public_id))
        } else {
            Entry::Vacant(NodeEntry::new(self, public_id))
        }
    }

    pub fn available_nodes(&self) -> &BTreeMap<Id, NodeRef> {
        &self.available
    }

    pub fn quarantined_nodes(&self) -> &HashMap<Id, NodeRef> {
        &self.quarantined
    }

    fn insert(&mut self, node: NodeRef) -> Option<NodeRef> {
        let public_id = *node.node().profile().public_id();
        self.available.insert(public_id, node.clone());
        self.all.insert(public_id, node.clone())
    }
}

impl<'a> NodeEntry<'a> {
    fn new(nodes: &'a mut Nodes, public_id: Id) -> Self {
        NodeEntry { nodes, public_id }
    }

    pub(crate) fn insert(&mut self, default: NodeRef) {
        debug_assert_eq!(self.key(), default.node().profile().public_id());
        assert!(self.nodes.insert(default).is_none())
    }

    fn key(&self) -> &Id {
        &self.public_id
    }

    pub(crate) fn release_mut(self) -> &'a mut NodeRef {
        self.nodes.all.get_mut(&self.public_id).unwrap()
    }

    pub(crate) fn modify<P, T, F>(&mut self, policy: &mut P, f: F) -> T
    where
        F: FnOnce(&mut NodeRef) -> T,
        P: Policy,
    {
        let node = self.nodes.all.get_mut(&self.public_id).unwrap();
        let result = f(node);

        match policy.check(node) {
            PolicyReport::None => {}
            PolicyReport::Forget => {
                self.nodes.available.remove(&self.public_id);
                self.nodes.quarantined.remove(&self.public_id);
                self.nodes.all.remove(&self.public_id);
            }
            PolicyReport::Quarantine => {
                self.nodes.available.remove(&self.public_id);
                self.nodes.quarantined.insert(self.public_id, node.clone());
                node.node_mut().logs_mut().quarantine();
            }
            PolicyReport::LiftQuarantine => {
                self.nodes.available.insert(self.public_id, node.clone());
                self.nodes.quarantined.remove(&self.public_id);
                node.node_mut().logs_mut().lift_quarantine();
            }
        }

        result
    }
}

impl<'a> Entry<'a> {
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: NodeRef) -> &'a mut NodeRef {
        let node_entry = match self {
            Entry::Vacant(mut node_entry) => {
                node_entry.insert(default);
                node_entry
            }
            Entry::Occupied(node_entry) => node_entry,
        };

        node_entry.release_mut()
    }

    /// Ensures a value is in the entry by inserting the result of the default function
    /// if empty, and returns a mutable reference to the value in the entry
    ///
    /// The advantage of this function over `or_insert` is that it is called only if
    /// the field was vacant
    pub fn or_insert_with<F>(self, default: F) -> &'a mut NodeRef
    where
        F: FnOnce() -> NodeRef,
    {
        let node_entry = match self {
            Entry::Vacant(mut node_entry) => {
                node_entry.insert(default());
                node_entry
            }
            Entry::Occupied(node_entry) => node_entry,
        };

        node_entry.release_mut()
    }

    pub fn key(&self) -> &Id {
        match self {
            Entry::Vacant(node_entry) => node_entry.key(),
            Entry::Occupied(node_entry) => node_entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential
    /// inserts into the collection
    pub fn and_modify<P, F>(self, policy: &mut P, f: F) -> Self
    where
        F: FnOnce(&mut NodeRef),
        P: Policy,
    {
        match self {
            Entry::Occupied(mut node_entry) => {
                node_entry.modify(policy, f);
                Entry::Occupied(node_entry)
            }
            entry => entry,
        }
    }
}
