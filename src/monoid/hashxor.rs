use std::marker::PhantomData;

use sha2::{Digest, Sha256};

use crate::{proto::ProtocolMonoid, LiftingMonoid};
use std::io::Write;

use super::FormattingMonoid;

#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub struct HashXorSha256<T>(usize, [u8; 32], PhantomData<T>)
where
    T: Eq + core::fmt::Debug + Clone + PartialOrd + Ord;

impl<T> LiftingMonoid for HashXorSha256<T>
where
    T: Eq + core::fmt::Debug + Clone + PartialOrd + Ord,
{
    type Item = T;

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

impl<T> FormattingMonoid for HashXorSha256<T>
where
    T: Eq + core::fmt::Debug + Clone + PartialOrd + Ord,
{
    fn item_to_string(item: &Self::Item) -> String {
        format!("{item:?}")
    }
}

impl<T> ProtocolMonoid for HashXorSha256<T>
where
    T: Eq + core::fmt::Debug + Clone + PartialOrd + Ord,
{
    fn count(&self) -> usize {
        let Self(count, _, _) = self;
        *count
    }
}
