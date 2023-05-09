use std::fmt::Debug;
use std::io::Write;
use std::marker::PhantomData;

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{
    monoid::{Item, Monoid},
    proto::ProtocolMonoid,
};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct CountingSha256Xor<I: Item>(usize, [u8; 32], PhantomData<I>);

impl<I: Item + Serialize> ProtocolMonoid for CountingSha256Xor<I>
where
    I: Clone + Debug + PartialOrd + Ord,
{
    type ProtocolItem = I;

    fn count(&self) -> usize {
        let Self(count, _, _) = self;
        *count
    }
}

impl<I: Item> Monoid for CountingSha256Xor<I> {
    type Item = I;

    fn neutral() -> Self {
        Self(0, [0; 32], PhantomData)
    }

    fn lift(item: &Self::Item) -> Self {
        let mut hasher = Sha256::default();
        write!(hasher, "{item:?}").expect("error hashing");
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
