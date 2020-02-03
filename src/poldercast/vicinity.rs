use crate::{GossipsBuilder, Id, Layer, Node, NodeProfile, Nodes, ViewBuilder};
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
    view: Vec<Id>,
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
                .par_iter()
                .filter(|id| *id != identity.id())
                .filter_map(|id| all_nodes.get(id))
                .collect(),
            VICINITY_MAX_VIEW_SIZE,
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
                .get(gossips_builder.recipient())
                .unwrap()
                .profile(),
            all_nodes
                .available_nodes()
                .par_iter()
                .filter(|id| *id != gossips_builder.recipient())
                .filter_map(|id| all_nodes.get(id))
                .collect(),
            VICINITY_MAX_GOSSIP_LENGTH,
        );
        for gossip in gossips {
            gossips_builder.add(gossip);
        }
    }

    fn view(&mut self, view_builder: &mut ViewBuilder, all_nodes: &mut Nodes) {
        for id in self.view.iter() {
            if let Some(node) = all_nodes.get_mut(id) {
                view_builder.add(node)
            }
        }
    }
}
impl Vicinity {
    /// select nodes based on the proximity function (see Profile's proximity
    /// member function).
    fn select_closest_nodes(
        &self,
        to: &NodeProfile,
        mut profiles: Vec<&Node>,
        max: usize,
    ) -> Vec<Id> {
        // Use unstable parallel sort as total number of nodes can be quite large.
        profiles.par_sort_unstable_by(|left, right| {
            to.proximity(left.profile())
                .cmp(&to.proximity(right.profile()))
        });

        profiles
            .into_iter()
            .take(max)
            .map(|v| v.id())
            .copied()
            .collect()
    }
}

impl Default for Vicinity {
    fn default() -> Self {
        Vicinity {
            view: Vec::default(),
        }
    }
}
