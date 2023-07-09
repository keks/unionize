extern crate alloc;
use alloc::format;

use core::cmp::Ordering;

use crate::protocol::SerializableItem;

use super::Item;

use serde::{Deserialize, Deserializer, Serialize};

/// Implements [`Ord`] for byte slices. Compares in little endian byte order.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct LEByteArray<const L: usize>(pub [u8; L]);

impl<const L: usize> ::core::fmt::Debug for LEByteArray<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if !f.alternate() {
            let mut flipped = [0u8; 4];
            for i in 0..4 {
                flipped[i] = self.0[L - 1 - i];
            }

            let hex_str = hex::encode(&flipped);

            write!(f, "LE_{hex_str}")
        } else {
            let mut flipped = [0u8; L];
            for i in 0..L {
                flipped[i] = self.0[L - 1 - i];
            }

            let hex_str = hex::encode(&flipped);

            write!(f, "LE_{hex_str}")
        }
    }
}

impl<const L: usize> PartialOrd for LEByteArray<L> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in (0..L).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Less => return Some(Ordering::Less),
                Ordering::Greater => return Some(Ordering::Greater),
                Ordering::Equal => {}
            }
        }

        Some(Ordering::Equal)
    }
}

impl<const L: usize> Ord for LEByteArray<L> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..L).rev() {
            match self.0[i].cmp(&other.0[i]) {
                Ordering::Less => return Ordering::Less,
                Ordering::Greater => return Ordering::Greater,
                Ordering::Equal => {}
            }
        }

        Ordering::Equal
    }
}

impl<const L: usize> Item for LEByteArray<L> {
    fn zero() -> Self {
        LEByteArray([0u8; L])
    }

    fn next(&self) -> Self {
        let mut result: Self = self.clone();
        for i in 0..L {
            let (sum, did_overflow) = result.0[i].overflowing_add(1);
            result.0[i] = sum;
            if !did_overflow {
                break;
            }
        }
        result
    }
}

impl<const L: usize> Serialize for LEByteArray<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<const L: usize> SerializableItem for LEByteArray<L> {}

impl<'de, const L: usize> Deserialize<'de> for LEByteArray<L> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(ArrayVisitor([(); L]))
    }
}

struct ArrayVisitor<const L: usize>([(); L]);

impl<'de, const L: usize> serde::de::Visitor<'de> for ArrayVisitor<L> {
    type Value = LEByteArray<L>;
    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!("{L} bytes/u8s"))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value.try_into() {
            Ok(buf) => Ok(LEByteArray(buf)),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(value),
                &self,
            )),
        }
    }
}

impl<const L: usize> Default for LEByteArray<L> {
    fn default() -> Self {
        LEByteArray([0u8; L])
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    use super::*;

    use proptest::{prop_assert_eq, proptest};

    proptest! {
        #[test]
        fn serialize_correctness(data in proptest::array::uniform30(0u8..=255u8)) {
            println!("d:{data:x?}");
            let item = LEByteArray(data);
            let encoded = serde_cbor::to_vec(&item).unwrap();
            println!("e:{encoded:x?}");
            let result = serde_cbor::from_slice(&encoded).unwrap();
            prop_assert_eq!(item, result);
        }
    }
}
