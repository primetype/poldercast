use crate::{Id, Node, Policy, PolicyReport};
use std::collections::{BTreeSet, HashMap, HashSet};

#[derive(Default, Debug)]
pub struct Nodes {
    all: HashMap<Id, Node>,

    quarantined: HashSet<Id>,

    available: BTreeSet<Id>,
}

pub enum Entry<'a> {
    Vacant(VacantEntry<'a>),
    Occupied(OccupiedEntry<'a>),
}

pub struct VacantEntry<'a> {
    nodes: &'a mut Nodes,
    id: Id,
}

pub struct OccupiedEntry<'a> {
    id: Id,
    nodes: &'a mut Nodes,
}

impl Nodes {
    pub(crate) fn get<'a>(&'a self, id: &Id) -> Option<&'a Node> {
        self.all.get(id)
    }

    pub(crate) fn get_mut<'a>(&'a mut self, id: &Id) -> Option<&'a mut Node> {
        self.all.get_mut(id)
    }

    pub fn entry(&mut self, public_id: Id) -> Entry<'_> {
        if self.all.contains_key(&public_id) {
            Entry::Occupied(OccupiedEntry::new(self, public_id))
        } else {
            Entry::Vacant(VacantEntry::new(self, public_id))
        }
    }

    pub fn available_nodes(&self) -> &BTreeSet<Id> {
        &self.available
    }

    pub fn quarantined_nodes(&self) -> &HashSet<Id> {
        &self.quarantined
    }

    fn insert(&mut self, node: Node) -> Option<Node> {
        let id = *node.id();
        self.available.insert(id);
        self.all.insert(id, node.clone())
    }
}

impl<'a> VacantEntry<'a> {
    fn new(nodes: &'a mut Nodes, id: Id) -> Self {
        VacantEntry { nodes, id }
    }

    pub(crate) fn insert(&mut self, default: Node) {
        debug_assert_eq!(self.key(), default.id());
        assert!(self.nodes.insert(default).is_none());
    }

    fn key(&self) -> &Id {
        &self.id
    }
}

impl<'a> OccupiedEntry<'a> {
    fn new(nodes: &'a mut Nodes, id: Id) -> Self {
        OccupiedEntry { nodes, id }
    }

    fn key(&self) -> &Id {
        &self.id
    }

    pub(crate) fn modify<P, F>(&mut self, policy: &mut P, f: F) -> PolicyReport
    where
        F: FnOnce(&mut Node),
        P: Policy,
    {
        let node = self.nodes.all.get_mut(&self.id).unwrap();
        f(node);
        let report = policy.check(node);

        match report {
            PolicyReport::None => {}
            PolicyReport::Forget => {
                self.nodes.available.remove(&self.id);
                self.nodes.quarantined.remove(&self.id);
                self.nodes.all.remove(&self.id);
            }
            PolicyReport::Quarantine => {
                self.nodes.available.remove(&self.id);
                self.nodes.quarantined.insert(self.id);
                node.logs_mut().quarantine();
            }
            PolicyReport::LiftQuarantine => {
                self.nodes.available.insert(self.id);
                self.nodes.quarantined.remove(&self.id);
                node.logs_mut().lift_quarantine();
            }
        }

        report
    }
}

impl<'a> Entry<'a> {
    /// Ensures a value is in the entry by inserting the default if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: Node) {
        match self {
            Entry::Vacant(mut node_entry) => node_entry.insert(default),
            Entry::Occupied(_node_entry) => {}
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function
    /// if empty, and returns a mutable reference to the value in the entry
    ///
    /// The advantage of this function over `or_insert` is that it is called only if
    /// the field was vacant
    pub fn or_insert_with<F>(self, default: F)
    where
        F: FnOnce() -> Node,
    {
        match self {
            Entry::Vacant(mut node_entry) => node_entry.insert(default()),
            Entry::Occupied(_node_entry) => {}
        }
    }

    pub fn key(&self) -> &Id {
        match self {
            Entry::Vacant(node_entry) => node_entry.key(),
            Entry::Occupied(node_entry) => node_entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any potential
    /// inserts into the collection
    pub fn and_modify<P, F>(self, policy: &mut P, f: F) -> Option<PolicyReport>
    where
        F: FnOnce(&mut Node),
        P: Policy,
    {
        match self {
            Entry::Occupied(mut node_entry) => Some(node_entry.modify(policy, f)),
            Entry::Vacant(_) => None,
        }
    }
}
