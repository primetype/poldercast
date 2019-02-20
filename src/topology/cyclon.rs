use rand_core::RngCore;
use std::collections::BTreeMap;

use crate::{topology::Module, Id, Node};

pub const CYCLON_MAX_GOSSIPING_LENGTH: usize = 128;

/// this module is responsible for randomly selecting Nodes
/// to be gossiped to another node.
///
/// It also make sure we contact the least contacted node for the next
/// gossiping round.
#[derive(Clone, Debug)]
pub struct Cyclon;
impl Cyclon {
    /// create the original Cyclon module, it will be responsible to
    /// maintain the view of all the connected peers for the other
    /// modules.
    pub fn new() -> Self {
        Cyclon
    }

    fn select_random_gossips<'a, Rng>(
        &self,
        rng: &mut Rng,
        known_nodes: &'a BTreeMap<Id, Node>,
    ) -> Vec<(&'a Id, &'a Node)>
    where
        Rng: RngCore,
    {
        let mut randomly_ordered_nodes: Vec<_> = known_nodes.iter().collect();
        randomly_ordered_nodes.sort_by(|_, _| match rng.next_u32() % 3 {
            0 => std::cmp::Ordering::Less,
            1 => std::cmp::Ordering::Equal,
            _ => std::cmp::Ordering::Greater,
        });

        randomly_ordered_nodes
    }
}

impl Module for Cyclon {
    fn name(&self) -> &'static str {
        "cyclon"
    }

    fn select_gossips(
        &self,
        our_node: &Node,
        gossip_recipient: &Node,
        known_nodes: &BTreeMap<Id, Node>,
    ) -> BTreeMap<Id, Node> {
        let mut candidates = BTreeMap::new();
        let mut rng = rand_os::OsRng::new().unwrap();
        candidates.insert(*our_node.id(), our_node.clone());

        candidates.extend(
            self.select_random_gossips(&mut rng, known_nodes)
                .into_iter()
                .filter(|(k, _)| k != &gossip_recipient.id())
                .take(CYCLON_MAX_GOSSIPING_LENGTH)
                .map(|(k, v)| (*k, v.clone())),
        );

        candidates
    }

    fn update(&mut self, _: &Node, _: &BTreeMap<Id, Node>) {
        /* nothing to update here, because we take all the known_nodes for the cyclon module */
    }

    fn view<'a>(&self, known_nodes: &'a BTreeMap<Id, Node>) -> BTreeMap<Id, &'a Node> {
        let mut node_iterator = known_nodes.values();
        let candidate = if let Some(candidate) = node_iterator.next() {
            candidate
        } else {
            return BTreeMap::new();
        };

        let candidate = node_iterator.fold(candidate, |candidate, prospect| {
            if candidate.last_gossip < prospect.last_gossip {
                candidate
            } else {
                prospect
            }
        });
        let mut candidates = BTreeMap::new();
        candidates.insert(*candidate.id(), candidate);
        candidates
    }
}
