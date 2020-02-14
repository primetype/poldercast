use crate::Node;
use serde::{Deserialize, Serialize};
use std::ops::Mul;
use std::{
    collections::VecDeque,
    time::{Duration, SystemTime},
};

/// default quarantine duration is 30min
const DEFAULT_QUARANTINE_DURATION: Duration = Duration::from_secs(1800);

pub struct DefaultPolicy;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strike {
    when: SystemTime,
    reason: StrikeReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrikeReason {
    CannotConnect,
    InvalidPublicId,
    InvalidData,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Record {
    lifetime_strikes: u32,
    strikes: VecDeque<Strike>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyReport {
    None,
    Quarantine,
    LiftQuarantine,
    Forget,
}

pub trait Policy {
    fn check(&mut self, node: &mut Node) -> PolicyReport;
}

impl Default for DefaultPolicy {
    fn default() -> Self {
        DefaultPolicy
    }
}

impl<P: Policy + ?Sized> Policy for Box<P> {
    fn check(&mut self, node: &mut Node) -> PolicyReport {
        self.as_mut().check(node)
    }
}

impl Policy for DefaultPolicy {
    fn check(&mut self, node: &mut Node) -> PolicyReport {
        // if the node is already quarantined
        if let Some(since) = node.logs().quarantined() {
            let duration = since.elapsed().unwrap();
            let quarantine_duration =
                DEFAULT_QUARANTINE_DURATION.mul(node.record().lifetime_strikes);

            if duration < quarantine_duration {
                // the node still need to do some quarantine time
                PolicyReport::None
            } else if node.logs().last_update().elapsed().unwrap() < quarantine_duration {
                // the node has been quarantined long enough, check if it has been updated
                // while being quarantined (i.e. the node is still up and advertising itself
                // or others are still gossiping about it.)

                // the fact that this `Policy` does clean the records is a policy choice.
                // one could prefer to keep the record longers for future `check`.
                node.record_mut().clean_slate();
                PolicyReport::LiftQuarantine
            } else {
                // it appears the node was quarantine and is no longer active or gossiped
                // about, so we can forget it
                PolicyReport::Forget
            }
        } else if node.record().is_clear() {
            // if the record is clear, do nothing, leave the Node in the available nodes
            PolicyReport::None
        } else {
            // if the record is not `clear` then we quarantine the block for some time
            PolicyReport::Quarantine
        }
    }
}

impl Record {
    /// returns `true` if the record show that the `Node` is clear
    /// of any strike
    pub fn is_clear(&self) -> bool {
        self.strikes.is_empty()
    }

    pub fn clean_slate(&mut self) {
        self.strikes = VecDeque::default()
    }

    pub fn strike(&mut self, reason: StrikeReason) {
        self.lifetime_strikes += 1;
        self.strikes.push_back(Strike {
            when: SystemTime::now(),
            reason,
        })
    }
}
