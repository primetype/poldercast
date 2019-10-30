use crate::{Id, NodeRef, Topic};
use std::collections::BTreeMap;

pub enum Selection {
    Topic { topic: Topic },
    Any,
}

pub struct ViewBuilder {
    event_origin: Option<NodeRef>,

    selection: Selection,

    view: BTreeMap<Id, NodeRef>,
}

impl ViewBuilder {
    pub fn new(selection: Selection) -> Self {
        Self {
            event_origin: None,
            selection,
            view: BTreeMap::new(),
        }
    }

    pub fn with_origin(mut self, origin: NodeRef) -> Self {
        self.event_origin = Some(origin);
        self
    }

    pub fn origin(&self) -> Option<&NodeRef> {
        self.event_origin.as_ref()
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    pub fn add(&mut self, node: NodeRef) {
        if let Selection::Topic { topic } = self.selection() {
            node.node_mut().logs_mut().use_of(*topic);
        }

        self.view.insert(node.public_id(), node);
    }

    pub fn build(self) -> Vec<NodeRef> {
        self.view
            .into_iter()
            .map(|(_, node_ref)| node_ref)
            .collect()
    }
}
