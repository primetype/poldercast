use crate::{Address, GossipsBuilder, Layer, NodeProfile, Nodes, ViewBuilder};
use rand::{seq::IteratorRandom, Rng};
use std::collections::BTreeSet;

const CYCLON_MAX_VIEW_LENGTH: usize = 20;
const CYCLON_MAX_GOSSIPING_LENGTH: usize = 10;

/// this module is responsible for randomly selecting Nodes
/// to be gossiped to another node.
///
/// It also make sure we contact the least contacted node for the next
/// gossiping round.
#[derive(Clone, Debug)]
pub struct Cyclon {
    view: Vec<Address>,
    max_view_length: usize,
    max_gossip_length: usize,
}

impl Cyclon {
    pub fn new(max_view_length: usize, max_gossip_length: usize) -> Self {
        Self {
            view: Vec::with_capacity(max_view_length),
            max_view_length,
            max_gossip_length,
        }
    }

    fn populate_random<R>(&mut self, mut rng: R, known_nodes: &BTreeSet<Address>, capacity: usize)
    where
        R: Rng,
    {
        self.view = known_nodes
            .iter()
            .map(|v| v)
            .cloned()
            .choose_multiple(&mut rng, capacity);
    }
}

impl Default for Cyclon {
    fn default() -> Self {
        Self::new(CYCLON_MAX_VIEW_LENGTH, CYCLON_MAX_GOSSIPING_LENGTH)
    }
}

impl Layer for Cyclon {
    fn alias(&self) -> &'static str {
        "cyclon"
    }

    fn reset(&mut self) {
        self.view.clear()
    }

    fn populate(&mut self, _identity: &NodeProfile, all_nodes: &Nodes) {
        self.populate_random(
            rand::thread_rng(),
            all_nodes.available_nodes(),
            self.max_view_length,
        )
    }

    fn gossips(
        &mut self,
        _identity: &NodeProfile,
        gossips_builder: &mut GossipsBuilder,
        all_nodes: &Nodes,
    ) {
        let mut cyclon = Cyclon::new(self.max_gossip_length, 0);
        cyclon.populate_random(
            rand::thread_rng(),
            all_nodes.available_nodes(),
            self.max_gossip_length,
        );

        cyclon.view.into_iter().for_each(|node| {
            gossips_builder.add(node);
        })
    }

    fn view(&mut self, view_builder: &mut ViewBuilder, all_nodes: &mut Nodes) {
        for id in self.view.iter() {
            if let Some(node) = all_nodes.peek_mut(id) {
                view_builder.add(node)
            }
        }
    }
}
