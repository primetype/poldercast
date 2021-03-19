#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod gossip;
pub mod layer;
mod priority_map;
mod profile;
mod profiles;
mod topic;
mod topology;

pub use self::{
    gossip::{Gossip, GossipError, GossipSlice},
    priority_map::PriorityMap,
    profile::Profile,
    profiles::Profiles,
    topic::{
        InterestLevel, Subscription, SubscriptionError, SubscriptionIter, SubscriptionSlice,
        Subscriptions, SubscriptionsSlice, Topic,
    },
    topology::Topology,
};
