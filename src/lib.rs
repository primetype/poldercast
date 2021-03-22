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

pub(crate) use self::profiles::Profiles;
pub use self::{
    gossip::{Gossip, GossipError, GossipSlice},
    priority_map::PriorityMap,
    profile::Profile,
    topic::{
        InterestLevel, Subscription, SubscriptionError, SubscriptionIter, SubscriptionSlice,
        Subscriptions, SubscriptionsSlice, Topic,
    },
    topology::Topology,
};
