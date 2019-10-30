#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod address;
mod gossip;
mod id;
mod layer;
mod logs;
mod node;
mod nodes;
pub mod poldercast;
mod policy;
mod topic;
mod topology;
mod view;

pub use self::{
    address::Address,
    gossip::{Gossip, Gossips, GossipsBuilder},
    id::Id,
    layer::Layer,
    logs::Logs,
    node::{Node, NodeProfile, NodeRef},
    nodes::Nodes,
    policy::{DefaultPolicy, Policy, PolicyReport, Record, Strike, StrikeReason},
    topic::{InterestLevel, Proximity, Subscription, Subscriptions, Topic},
    topology::{GossipingError, Topology},
    view::{Selection, ViewBuilder},
};
