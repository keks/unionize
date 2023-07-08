use crate::item::Item;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct Range<T: Item>(pub(crate) T, pub(crate) T);

impl<T: Item> Range<T> {
    pub fn reverse(&self) -> Self {
        let Self(from, to) = self;
        Self(to.clone(), from.clone())
    }

    #[inline]
    pub fn from(&self) -> &T {
        &self.0
    }

    #[inline]
    pub fn to(&self) -> &T {
        &self.1
    }

    #[inline]
    pub(crate) fn is_wrapping(&self) -> bool {
        let Range(from, to) = self;
        from >= to
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        let Range(from, to) = self;
        from == to
    }

    #[inline]
    pub(crate) fn contains(&self, item: &T) -> bool {
        let Range(from, to) = self;
        let wrapping_case = from <= item || item < to;
        let non_wrapping_case = from <= item && item < to;
        let is_wrapping = self.is_wrapping();

        (is_wrapping && wrapping_case) || non_wrapping_case
        // equivalent to:
        // if self.is_wrapping() {
        //     wrapping_case
        // } else {
        //     non_wrapping_case
        // }
    }

    #[inline]
    pub(crate) fn partially_contains(&self, min: &T, max: &T) -> bool {
        let node_bounds_around_query_range = min < self.from() && self.to() <= max;

        self.contains(min) || self.contains(max) || node_bounds_around_query_range
    }

    #[inline]
    pub(crate) fn fully_contains(&self, min: &T, max: &T) -> bool {
        let Range(from, to) = self;
        let is_full = self.is_full();
        let is_wrapping = self.is_wrapping();

        let wrapping_case = from <= min || max < to;
        let non_wrapping_case = from <= min && max < to;

        is_full || (is_wrapping && wrapping_case) || non_wrapping_case
        // equivalent to:
        // if self.is_full() {
        //     true
        // } else if self.is_wrapping() {
        //     wrapping_case
        // } else {
        //     non_wrapping_case
        // }
    }
}

impl<T: Item + Copy> Copy for Range<T> {}

impl<T: Item> core::fmt::Display for Range<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self(from, to) = self;
        write!(f, "{from:?}..{to:?}")
    }
}

// impl<I: Item + Serialize> Serialize for Range<I> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut tup = serializer.serialize_tuple(2)?;
//         tup.serialize_element(&self.0)?;
//         tup.serialize_element(&self.1)?;
//         tup.end()
//     }
// }

// impl<'de, I: Item + Deserialize<'de>> Deserialize<'de> for Range<I> {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let mut tup = deserializer.deserialize_tuple(2)?;
//
//         tup.
//         let from = I::deserialize()?;
//         let to = I::deserialize(deserializer)?;
//
//         Ok(Range(from, to))
//     }
// }

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::format;

    use super::*;
    use crate::item::le_byte_array::LEByteArray;

    use proptest::{prop_assert_eq, proptest};

    proptest! {
        #[test]
        fn serialize_correctness(from in proptest::array::uniform30(0u8..=255u8), to in proptest::array::uniform30(0u8..=255u8)) {
            let from_item = LEByteArray(from);
            let to_item = LEByteArray(to);
            let range = Range(from_item, to_item);
            let encoded = serde_cbor::to_vec(&range).unwrap();
            let result = serde_cbor::from_slice(&encoded).unwrap();
            prop_assert_eq!(range, result);
        }
    }
}
