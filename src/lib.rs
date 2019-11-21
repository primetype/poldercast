/**
# Overview

Poldercast is a multilayer based topology builder. It helps interconnect nodes
in a peer to peer decentralized way. This _crate_ does not do the connection part.
It only provides a way to construct the topology and how to communicate between
nodes.

This `crate` has been implemented based on the work Vinay Setty, Maarten van Steen,
Roman Vitenberg and Spyros Voulgaris: [PolderCast: Fast, Robust and Scalable
Architecture for P2P Topic-based Pub/Sub](https://hal.inria.fr/hal-01555561).
However there are some slight differences in order to adapt the protocol based on
the needs.

*/

#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

mod address;
pub mod custom_layers;
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
    gossip::{Gossips, GossipsBuilder},
    id::Id,
    layer::Layer,
    logs::Logs,
    node::{Node, NodeInfo, NodeProfile, NodeProfileBuilder},
    nodes::Nodes,
    policy::{DefaultPolicy, Policy, PolicyReport, Record, Strike, StrikeReason},
    topic::{InterestLevel, Proximity, Subscription, Subscriptions, Topic},
    topology::Topology,
    view::{Selection, ViewBuilder},
};
