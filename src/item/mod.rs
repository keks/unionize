pub mod byte_slice;
pub mod le_byte_array;
pub mod timestamped;
pub mod tuple;
pub mod uint;

/// The item that should be replicated. Since this protocol
/// uses ranges, it need to be ordered.
/// In many cases, these will be hashes to the actual data transmitted,
/// but it may also contain metadata in order to make the protocol more
/// efficient.
pub trait Item: Clone + Ord + core::fmt::Debug {
    /// Returns the lowest possible item.
    fn zero() -> Self;

    /// Returns the lowest item greater than `self`.
    /// For numbers, this is `self + 1`.
    fn next(&self) -> Self;
}
