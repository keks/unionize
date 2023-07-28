extern crate alloc;
use alloc::{collections::BTreeMap, vec::Vec};

pub mod timestamped;

use crate::Item;

pub trait Object<I>: Clone + core::fmt::Debug {
    fn to_item(&self) -> I;
    fn validate_self_consistency(&self) -> bool;
}

pub trait ObjectStore<I, O>
where
    I: Item,
    O: Object<I>,
{
    fn get(&self, item: &I) -> Option<&O>;

    fn get_batch(&self, items: &[I]) -> Vec<Option<&O>> {
        items.iter().map(|item| self.get(item)).collect()
    }
}

impl<I, O> ObjectStore<I, O> for BTreeMap<I, O>
where
    I: Item,
    O: Object<I>,
{
    fn get(&self, item: &I) -> Option<&O> {
        self.get(item)
    }
}

impl<T: Clone + core::fmt::Debug> Object<T> for (T, bool) {
    fn to_item(&self) -> T {
        self.0.clone()
    }

    fn validate_self_consistency(&self) -> bool {
        self.1
    }
}
