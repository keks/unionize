pub mod byte_slice;
pub mod le_byte_array;
pub mod uint;

pub trait Item: Clone + Ord + std::fmt::Debug {
    fn zero() -> Self;
    fn next(&self) -> Self;
}