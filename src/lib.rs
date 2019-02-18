#[cfg(test)]
#[macro_use]
extern crate quickcheck;

mod topic;

mod node;
pub mod topology;

pub use self::node::{Address, Id, Node};
pub use self::topic::{InterestLevel, Proximity, Subscription, Subscriptions, Topic};
