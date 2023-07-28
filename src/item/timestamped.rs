use serde::{Deserialize, Serialize};

use crate::Item;

pub trait TimestampItem: Item {}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TimestampedItem<TS: TimestampItem, I: Item>(pub TS, pub I);

impl<TS: TimestampItem, I: Item> TimestampedItem<TS, I> {
    pub fn new(ts: TS, lower: I) -> Self {
        Self(ts, lower)
    }

    pub fn as_timestamp(&self) -> &TS {
        &self.0
    }

    pub fn as_lower(&self) -> &I {
        &self.1
    }
}

impl<TS: TimestampItem, I: Item> Item for TimestampedItem<TS, I> {
    fn zero() -> Self {
        Self(TS::zero(), I::zero())
    }

    fn next(&self) -> Self {
        Self(self.0.clone(), self.1.next())
    }
}
