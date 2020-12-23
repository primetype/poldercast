mod cyclon;
mod rings;
mod vicinity;

pub use self::{cyclon::Cyclon, rings::Rings, vicinity::Vicinity};
use crate::{InterestLevel, PriorityMap, Profile, Topic};
use keynesis::key::ed25519;
use std::collections::HashSet;

pub trait Layer {
    fn name(&self) -> &'static str;

    fn view(&mut self, builder: &mut ViewBuilder);

    fn remove(&mut self, id: &ed25519::PublicKey);
    fn reset(&mut self);

    fn subscribe(&mut self, topic: Topic);
    fn unsubscribe(&mut self, topic: &Topic);
    fn subscriptions(&self, output: &mut PriorityMap<InterestLevel, Topic>);

    fn populate(&mut self, our_profile: &Profile, new_profile: &Profile);
}

pub trait LayerBuilder {
    fn build_for_view(&self) -> Vec<Box<dyn Layer>>;
    fn build_for_gossip(&self) -> Vec<Box<dyn Layer>>;
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Selection {
    Topic { topic: Topic },
    Any,
}

#[doc(hidden)]
pub struct ViewBuilder {
    event_origin: Option<ed25519::PublicKey>,

    selection: Selection,

    view: HashSet<ed25519::PublicKey>,
}

impl ViewBuilder {
    pub fn new(selection: Selection) -> Self {
        Self {
            event_origin: None,
            selection,
            view: HashSet::new(),
        }
    }

    pub fn with_origin(&mut self, origin: ed25519::PublicKey) -> &Self {
        self.event_origin = Some(origin);
        self
    }

    pub fn origin(&self) -> Option<&ed25519::PublicKey> {
        self.event_origin.as_ref()
    }

    pub fn selection(&self) -> Selection {
        self.selection
    }

    pub fn add(&mut self, node: &ed25519::PublicKey) {
        self.view.insert(*node);
    }

    pub(crate) fn build(self) -> HashSet<ed25519::PublicKey> {
        self.view
    }
}
