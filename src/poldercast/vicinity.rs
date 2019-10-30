use crate::{GossipsBuilder, Id, Layer, NodeProfile, NodeRef, Nodes, ViewBuilder};
use std::collections::BTreeMap;

const VICINITY_MAX_VIEW_SIZE: usize = 20;
const VICINITY_MAX_GOSSIP_LENGTH: usize = 10;

/// The Vicinity module is responsible for maintaining interest-induced
/// random links, that is, randomly chosen links between nodes that share
/// one or more topics. Such links serve as input to the Rings module.
/// Additionally, they are used by the dissemination protocol to propagate
/// events to arbitrary subscribers of a topic.
#[derive(Clone, Debug)]
pub struct Vicinity {
    view: Vec<NodeRef>,
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
            all_nodes.available_nodes(),
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
            gossips_builder.recipient().node().profile(),
            all_nodes.available_nodes(),
            VICINITY_MAX_GOSSIP_LENGTH,
        );
        for gossip in gossips {
            gossips_builder.add(gossip);
        }
    }

    fn view(&mut self, view_builder: &mut ViewBuilder) {
        self.view
            .iter()
            .for_each(|node| view_builder.add(node.clone()));
    }
}
impl Vicinity {
    /// select nodes based on the proximity function (see Profile's proximity
    /// member function).
    fn select_closest_nodes(
        &self,
        to: &NodeProfile,
        known_nodes: &BTreeMap<Id, NodeRef>,
        max: usize,
    ) -> Vec<NodeRef> {
        let mut profiles: Vec<_> = known_nodes.values().collect();

        profiles.sort_by(|left, right| {
            to.proximity(left.node().profile())
                .cmp(&to.proximity(right.node().profile()))
        });

        profiles.into_iter().take(max).cloned().collect()
    }
}

impl Default for Vicinity {
    fn default() -> Self {
        Vicinity {
            view: Vec::default(),
        }
    }
}
