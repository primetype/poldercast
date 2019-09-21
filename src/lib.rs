#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(feature = "serde_derive")]
#[macro_use(cfg_if)]
extern crate cfg_if;

mod topic;

mod node;
#[cfg(feature = "serde_derive")]
mod serde;
pub mod topology;

pub use self::node::{Address, Id, Node};
pub use self::topic::{InterestLevel, Proximity, Subscription, Subscriptions, Topic};
