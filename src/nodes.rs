use crate::{Address, Node, Policy, PolicyReport};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug)]
pub struct Nodes {
    all: LruCache<Address, Node>,
    quarantined: HashSet<Address>,
    not_reachable: HashSet<Address>,
    available: HashSet<Address>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Count {
    pub all_count: usize,
    pub quarantined_count: usize,
    pub not_reachable_count: usize,
    pub available_count: usize,
}

pub enum Entry<'a> {
    Vacant(VacantEntry<'a>),
    Occupied(OccupiedEntry<'a>),
}

pub struct VacantEntry<'a> {
    nodes: &'a mut Nodes,
    id: Address,
}

pub struct OccupiedEntry<'a> {
    id: Address,
    nodes: &'a mut Nodes,
}

impl Nodes {
    pub fn new(cap: usize) -> Self {
        Self {
            all: LruCache::new(cap),
            quarantined: HashSet::default(),
            not_reachable: HashSet::default(),
            available: HashSet::default(),
        }
    }

    pub(crate) fn get<'a>(&'a mut self, id: &Address) -> Option<&'a Node> {
        self.all.get(id)
    }

    pub(crate) fn get_mut<'a>(&'a mut self, id: &Address) -> Option<&'a mut Node> {
        self.all.get_mut(id)
    }

    pub(crate) fn peek<'a>(&'a self, id: &Address) -> Option<&'a Node> {
        self.all.peek(id)
    }

    pub(crate) fn peek_mut<'a>(&'a mut self, id: &Address) -> Option<&'a mut Node> {
        self.all.peek_mut(id)
    }

    pub fn entry(&mut self, public_id: Address) -> Entry<'_> {
        if self.all.contains(&public_id) {
            Entry::Occupied(OccupiedEntry::new(self, public_id))
        } else {
            Entry::Vacant(VacantEntry::new(self, public_id))
        }
    }

    pub fn available_nodes(&self) -> &HashSet<Address> {
        &self.available
    }

    /// list all available nodes, these are nodes that are not quarantined
    /// and that are publicly reachable.
    ///
    /// This operation is costly and should not be used often or it will slow
    /// down the other operation of the `Nodes`
    pub fn all_available_nodes(&self) -> Vec<&Node> {
        self.available_nodes()
            .iter()
            .filter_map(|id| self.all.peek(id))
            .collect()
    }

    /// list all quarantined nodes, these are nodes that are not in used in the
    /// p2p topology and but may become available or be removed soon.
    ///
    /// This operation is costly and should not be used often or it will slow
    /// down the other operation of the `Nodes`
    pub fn all_quarantined_nodes(&self) -> Vec<&Node> {
        self.quarantined_nodes()
            .iter()
            .filter_map(|id| self.all.peek(id))
            .collect()
    }

    /// list all non publicly reachable nodes. These are nodes that are directly
    /// connected to our nodes and that are not gossiped about.
    ///
    /// This operation is costly and should not be used often or it will slow
    /// down the other operation of the `Nodes`
    pub fn all_unreachable_nodes(&self) -> Vec<&Node> {
        self.unreachable_nodes()
            .iter()
            .filter_map(|id| self.all.peek(id))
            .collect()
    }

    /// access nodes that are connected to us but not necessarily reachable
    ///
    /// This can be nodes that are behind a firewall or a NAT and that can't do
    /// hole punching to allow other nodes to connect to them.
    pub fn unreachable_nodes(&self) -> &HashSet<Address> {
        &self.not_reachable
    }

    pub fn quarantined_nodes(&self) -> &HashSet<Address> {
        &self.quarantined
    }

    /// access a count of all nodes
    pub fn node_count(&self) -> Count {
        Count {
            all_count: self.all.len(),
            available_count: self.available.len(),
            not_reachable_count: self.not_reachable.len(),
            quarantined_count: self.quarantined.len(),
        }
    }

    fn insert(&mut self, node: Node) -> Option<Node> {
        use crate::node::NodeAddress::*;
        let address = match node.node_address() {
            Discoverable(address) => {
                self.available.insert(address.clone());
                address
            }
            NonDiscoverable(address) => {
                self.not_reachable.insert(address.clone());
                address
            }
        };
        self.all.put(address, node)
    }

    pub(crate) fn reset<P>(&mut self, policy: &mut P)
    where
        P: Policy,
    {
        let available = &mut self.available;
        let not_reachable = &mut self.not_reachable;
        let quarantined = &mut self.quarantined;

        let mut to_remove = Vec::new();

        for (k, node) in self.all.iter_mut() {
            let report = policy.check(node);

            match report {
                PolicyReport::None => (),
                PolicyReport::Forget => {
                    available.remove(k);
                    not_reachable.remove(k);
                    quarantined.remove(k);

                    to_remove.push(k.clone());
                }
                PolicyReport::Quarantine => {
                    available.remove(k);
                    not_reachable.remove(k);
                    quarantined.insert(k.clone());
                    node.logs_mut().quarantine();
                }
                PolicyReport::LiftQuarantine => {
                    if node.node_address().is_discoverable() {
                        available.insert(k.clone());
                    } else {
                        not_reachable.insert(k.clone());
                    }
                    quarantined.remove(k);
                    node.logs_mut().lift_quarantine();
                }
            }
        }

        for k in to_remove {
            self.all.pop(&k);
        }
    }
}

impl<'a> VacantEntry<'a> {
    fn new(nodes: &'a mut Nodes, id: Address) -> Self {
        VacantEntry { nodes, id }
    }

    pub(crate) fn insert(&mut self, default: Node) {
        debug_assert_eq!(self.key(), default.node_address().as_ref());
        assert!(self.nodes.insert(default).is_none());
    }

    fn key(&self) -> &Address {
        &self.id
    }
}

impl<'a> OccupiedEntry<'a> {
    fn new(nodes: &'a mut Nodes, id: Address) -> Self {
        OccupiedEntry { nodes, id }
    }

    fn key(&self) -> &Address {
        &self.id
    }

    pub(crate) fn modify<P, F>(&mut self, policy: &mut P, f: F) -> PolicyReport
    where
        F: FnOnce(&mut Node),
        P: Policy,
    {
        let node = self.nodes.all.get_mut(&self.id).unwrap();
        let was_reachable = node.node_address().is_discoverable();
        f(node);
        let report = policy.check(node);

        match report {
            PolicyReport::None => {
                let now_reachable = node.node_address().is_discoverable();
                if was_reachable && !now_reachable {
                    if self.nodes.available.remove(&self.id) {
                        self.nodes.not_reachable.insert(self.id.clone());
                    }
                } else if !was_reachable
                    && now_reachable
                    && self.nodes.not_reachable.remove(&self.id)
                {
                    self.nodes.available.insert(self.id.clone());
                }
            }
            PolicyReport::Forget => {
                self.nodes.available.remove(&self.id);
                self.nodes.not_reachable.remove(&self.id);
                self.nodes.quarantined.remove(&self.id);
                self.nodes.all.pop(&self.id);
            }
            PolicyReport::Quarantine => {
                self.nodes.available.remove(&self.id);
                self.nodes.not_reachable.remove(&self.id);
                self.nodes.quarantined.insert(self.id.clone());
                node.logs_mut().quarantine();
            }
            PolicyReport::LiftQuarantine => {
                if node.node_address().is_discoverable() {
                    self.nodes.available.insert(self.id.clone());
                } else {
                    self.nodes.not_reachable.insert(self.id.clone());
                }
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

    pub fn key(&self) -> &Address {
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
