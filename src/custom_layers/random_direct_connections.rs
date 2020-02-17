use crate::{GossipsBuilder, Id, Layer, NodeProfile, Nodes, ViewBuilder};
use rand::{seq::IteratorRandom, Rng};
use std::collections::HashSet;

const DEFAULT_MAX_VIEW_LENGTH: usize = 20;

/// module responsible to select some random nodes that are not publicly
/// reachable but that are directly connected to us.
///
/// This layer selects node for event propagation, but not for gossiping.
#[derive(Clone, Debug)]
pub struct RandomDirectConnections {
    view: Vec<Id>,
    max_view_length: usize,
}

impl RandomDirectConnections {
    /// create a `RandomDirectConnections` layer that will select some
    /// random nodes to propagate event. Nodes that are directly connected
    /// to our node but without being publicly reachable
    pub fn with_max_view_length(max_view_length: usize) -> Self {
        Self {
            view: Vec::with_capacity(max_view_length),
            max_view_length,
        }
    }

    fn populate_random<R>(&mut self, mut rng: R, known_nodes: &HashSet<Id>, capacity: usize)
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

impl Default for RandomDirectConnections {
    fn default() -> Self {
        RandomDirectConnections {
            view: Vec::with_capacity(DEFAULT_MAX_VIEW_LENGTH),
            max_view_length: DEFAULT_MAX_VIEW_LENGTH,
        }
    }
}

impl Layer for RandomDirectConnections {
    fn alias(&self) -> &'static str {
        "random_direct_connections"
    }

    fn reset(&mut self) {
        self.view.clear()
    }

    fn populate(&mut self, _identity: &NodeProfile, all_nodes: &Nodes) {
        self.populate_random(
            rand::thread_rng(),
            all_nodes.unreachable_nodes(),
            self.max_view_length,
        )
    }

    fn gossips(
        &mut self,
        _identity: &NodeProfile,
        _gossips_builder: &mut GossipsBuilder,
        _all_nodes: &Nodes,
    ) {
    }

    fn view(&mut self, view_builder: &mut ViewBuilder, all_nodes: &mut Nodes) {
        for id in self.view.iter() {
            if let Some(node) = all_nodes.peek_mut(id) {
                view_builder.add(node)
            }
        }
    }
}
