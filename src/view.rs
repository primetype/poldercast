use crate::{Address, Node, NodeInfo, Nodes, Topic};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    Topic { topic: Topic },
    Any,
}

pub struct ViewBuilder {
    event_origin: Option<Address>,

    selection: Selection,

    view: HashSet<Address>,
}

impl ViewBuilder {
    pub fn new(selection: Selection) -> Self {
        Self {
            event_origin: None,
            selection,
            view: HashSet::new(),
        }
    }

    pub fn with_origin(&mut self, origin: Address) -> &Self {
        self.event_origin = Some(origin);
        self
    }

    pub fn origin(&self) -> Option<&Address> {
        self.event_origin.as_ref()
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn add(&mut self, node: &mut Node) {
        if let Selection::Topic { topic } = self.selection() {
            node.logs_mut().use_of(*topic);
        }

        self.view.insert(node.address().as_ref().clone());
    }

    pub fn build(self, nodes: &Nodes) -> Vec<NodeInfo> {
        self.view
            .into_iter()
            .filter_map(|id| nodes.get(&id))
            .map(|node| node.info().clone())
            .collect()
    }
}
