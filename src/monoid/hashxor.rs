use core::marker::PhantomData;
use core::{convert::Infallible, fmt::Debug};

extern crate alloc;
use alloc::format;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::protocol::{DecodeError, EncodeError, SerializableItem};
use crate::{
    monoid::{Item, Monoid},
    protocol::{Encodable, ProtocolMonoid},
};

/// This monoid lifts by hashing the debug string of an item and combines by XORing.
/// Should probably only be used for tests.
/// One reason this is needed because if we tests by XORing simple numbers, collisions are very
/// likely.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct CountingSha256Xor<I: Item>(usize, [u8; 32], PhantomData<I>);

impl<I> ProtocolMonoid for CountingSha256Xor<I>
where
    I: SerializableItem,
{
    // type SerializableItem = I;

    fn count(&self) -> usize {
        let Self(count, _, _) = self;
        *count
    }
}

impl<I: Item> Default for CountingSha256Xor<I> {
    fn default() -> Self {
        CountingSha256Xor(0, [0u8; 32], PhantomData)
    }
}

impl<I: Item> Encodable for CountingSha256Xor<I>
where
    I: Clone + Debug + PartialOrd + Ord,
{
    type Encoded = Self;
    type EncodeError = Infallible;
    type DecodeError = Infallible;

    fn encode(&self, encoded: &mut Self::Encoded) -> Result<(), EncodeError<Self::EncodeError>> {
        *encoded = self.clone();
        Ok(())
    }

    fn decode(&mut self, encoded: &Self::Encoded) -> Result<(), DecodeError<Self::DecodeError>> {
        *self = encoded.clone();
        Ok(())
    }
}

impl<I: Item> Monoid for CountingSha256Xor<I> {
    type Item = I;

    fn neutral() -> Self {
        Self(0, [0; 32], PhantomData)
    }

    fn lift(item: &Self::Item) -> Self {
        let mut hasher = Sha256::default();
        hasher.update(&format!("{item:?}"));
        let hash = hasher.finalize();
        Self(1, hash.into(), PhantomData)
    }

    fn combine(&self, other: &Self) -> Self {
        let Self(left_count, left, _) = self;
        let Self(right_count, right, _) = other;

        let mut out = [0; 32];
        for i in 0..32 {
            out[i] = left[i] ^ right[i];
        }

        Self(left_count + right_count, out, PhantomData)
    }
}
