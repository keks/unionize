use serde::{Deserialize, Serialize};

use crate::{
    item::timestamped::{TimestampItem, TimestampedItem},
    Item, Object,
};

pub trait TimestampedObject: core::fmt::Debug + Clone {
    type Timestamp: TimestampItem + Serialize + for<'de2> Deserialize<'de2>;
    type Unique: Item;

    fn to_timestamp(&self) -> Self::Timestamp;
    fn to_unique(&self) -> Self::Unique;

    fn validate_self_consistency(&self) -> bool;
}

impl<O: TimestampedObject> Object<TimestampedItem<O::Timestamp, O::Unique>> for O {
    fn to_item(&self) -> TimestampedItem<O::Timestamp, O::Unique> {
        TimestampedItem(self.to_timestamp(), self.to_unique())
    }

    fn validate_self_consistency(&self) -> bool {
        self.validate_self_consistency()
    }
}
