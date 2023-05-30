use std::cmp::Ordering;

use crate::monoid::{Item, Peano};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct LEByteArray<const L: usize>(pub [u8; L]);

impl<const L: usize> ::core::fmt::Debug for LEByteArray<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flipped = [0u8; 4];
        for i in 0..4 {
            flipped[i] = self.0[L - 1 - i];
        }

        let hex_str = hex::encode(&flipped);

        write!(f, "LE_{hex_str}")
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

impl<const L: usize> Item for LEByteArray<L> {}

impl<const L: usize> Peano for LEByteArray<L> {
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
impl<const L: usize> Default for LEByteArray<L> {
    fn default() -> Self {
        LEByteArray([0u8; L])
    }
}
