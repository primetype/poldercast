use crate::{Address, GossipsBuilder, Layer, Node, NodeProfile, Nodes, ViewBuilder};
use rayon::prelude::*;

const VICINITY_MAX_VIEW_SIZE: usize = 20;
const VICINITY_MAX_GOSSIP_LENGTH: usize = 10;

/// The Vicinity module is responsible for maintaining interest-induced
/// random links, that is, randomly chosen links between nodes that share
/// one or more topics. Such links serve as input to the Rings module.
/// Additionally, they are used by the dissemination protocol to propagate
/// events to arbitrary subscribers of a topic.
#[derive(Clone, Debug)]
pub struct Vicinity {
    view: Vec<Address>,
    max_view_size: usize,
    max_gossip_length: usize,
}
impl Layer for Vicinity {
    fn alias(&self) -> &'static str {
        "vicinity"
    }

    fn reset(&mut self) {
        self.view.clear()
    }

    fn populate(&mut self, identity: &NodeProfile, all_nodes: &Nodes) {
        self.view = self.select_closest_nodes(
            identity,
            all_nodes
                .available_nodes()
                .iter()
                .filter(|id| Some(*id) != identity.address())
                .filter_map(|id| all_nodes.peek(id))
                .collect(),
            self.max_view_size,
        )
    }

    fn gossips(
        &mut self,
        _identity: &NodeProfile,
        gossips_builder: &mut GossipsBuilder,
        all_nodes: &Nodes,
    ) {
        let gossips = self.select_closest_nodes(
            all_nodes
                .peek(gossips_builder.recipient())
                .unwrap()
                .profile(),
            all_nodes
                .available_nodes()
                .iter()
                .filter(|id| *id != gossips_builder.recipient())
                .filter_map(|id| all_nodes.peek(id))
                .collect(),
            self.max_gossip_length,
        );
        for gossip in gossips {
            gossips_builder.add(gossip);
        }
    }

    fn view(&mut self, view_builder: &mut ViewBuilder, all_nodes: &mut Nodes) {
        for id in self.view.iter() {
            if let Some(node) = all_nodes.peek_mut(id) {
                view_builder.add(node)
            }
        }
    }
}
impl Vicinity {
    pub fn new(max_view_size: usize, max_gossip_length: usize) -> Self {
        Self {
            view: Vec::with_capacity(max_view_size),
            max_view_size,
            max_gossip_length,
        }
    }

    /// select nodes based on the proximity function (see Profile's proximity
    /// member function).
    fn select_closest_nodes(
        &self,
        to: &NodeProfile,
        mut profiles: Vec<&Node>,
        max: usize,
    ) -> Vec<Address> {
        // Use unstable parallel sort as total number of nodes can be quite large.
        profiles.par_sort_unstable_by(|left, right| {
            to.proximity(left.profile())
                .cmp(&to.proximity(right.profile()))
        });

        profiles
            .into_iter()
            .take(max)
            .map(|v| v.address().as_ref().clone())
            .collect()
    }
}

impl Default for Vicinity {
    fn default() -> Self {
        Vicinity {
            view: Vec::default(),
            max_view_size: VICINITY_MAX_VIEW_SIZE,
            max_gossip_length: VICINITY_MAX_GOSSIP_LENGTH,
        }
    }
}
