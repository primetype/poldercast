use crate::{GossipsBuilder, Id, Layer, NodeProfile, Nodes, ViewBuilder};
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
pub struct Cyclon(Vec<Id>);

impl Cyclon {
    fn with_capacity(capacity: usize) -> Self {
        Cyclon(Vec::with_capacity(capacity))
    }

    fn populate_random<R>(&mut self, mut rng: R, known_nodes: &BTreeSet<Id>, capacity: usize)
    where
        R: Rng,
    {
        self.0 = known_nodes
            .iter()
            .cloned()
            .choose_multiple(&mut rng, capacity);
    }
}

impl Default for Cyclon {
    fn default() -> Self {
        Self::with_capacity(CYCLON_MAX_VIEW_LENGTH)
    }
}

impl Layer for Cyclon {
    fn alias(&self) -> &'static str {
        "cyclon"
    }

    fn reset(&mut self) {
        self.0.clear()
    }

    fn populate(&mut self, _identity: &NodeProfile, all_nodes: &Nodes) {
        self.populate_random(
            rand::thread_rng(),
            all_nodes.available_nodes(),
            CYCLON_MAX_VIEW_LENGTH,
        )
    }

    fn gossips(
        &mut self,
        _identity: &NodeProfile,
        gossips_builder: &mut GossipsBuilder,
        all_nodes: &Nodes,
    ) {
        let mut cyclon = Cyclon::with_capacity(CYCLON_MAX_GOSSIPING_LENGTH);
        cyclon.populate_random(
            rand::thread_rng(),
            all_nodes.available_nodes(),
            CYCLON_MAX_GOSSIPING_LENGTH,
        );

        cyclon.0.into_iter().for_each(|node| {
            gossips_builder.add(node);
        })
    }

    fn view(&mut self, view_builder: &mut ViewBuilder, all_nodes: &mut Nodes) {
        for id in self.0.iter() {
            if let Some(node) = all_nodes.get_mut(id) {
                view_builder.add(node)
            }
        }
    }
}
