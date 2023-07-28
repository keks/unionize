use core::marker::PhantomData;

use crate::{
    item::timestamped::{TimestampItem, TimestampedItem},
    protocol::{Encodable, ProtocolMonoid},
    Monoid,
};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
pub struct Timestamped<TS: TimestampItem, M: Monoid>(M, PhantomData<TS>);

impl<TS: TimestampItem, M: Monoid> Monoid for Timestamped<TS, M> {
    type Item = TimestampedItem<TS, M::Item>;

    fn neutral() -> Self {
        Self(M::neutral(), PhantomData)
    }

    fn lift(item: &Self::Item) -> Self {
        Self(M::lift(item.as_lower()), PhantomData)
    }

    fn combine(&self, other: &Self) -> Self {
        Self(M::combine(&self.0, &other.0), PhantomData)
    }
}

impl<TS: TimestampItem, M: Default + Monoid> Default for Timestamped<TS, M> {
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}

impl<TS: TimestampItem, M: ProtocolMonoid> Encodable for Timestamped<TS, M> {
    type Encoded = M::Encoded;

    type EncodeError = M::EncodeError;

    type DecodeError = M::DecodeError;

    fn encode(
        &self,
        encoded: &mut Self::Encoded,
    ) -> Result<(), crate::protocol::EncodeError<Self::EncodeError>> {
        self.0.encode(encoded)
    }

    fn decode(
        &mut self,
        encoded: &Self::Encoded,
    ) -> Result<(), crate::protocol::DecodeError<Self::DecodeError>> {
        self.0.decode(encoded)
    }
}

impl<TS: TimestampItem, M: ProtocolMonoid> ProtocolMonoid for Timestamped<TS, M> {
    fn count(&self) -> usize {
        self.0.count()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        item::le_byte_array::LEByteArray,
        monoid::{count::CountingMonoid, mulhash_xs233::Xsk233MulHashMonoid},
    };

    use super::*;

    #[test]
    fn timestamped_is_protocolmonoid() {
        let item = TimestampedItem::new(0u64, LEByteArray::<30>::default());
        let monoid = Timestamped::<u64, CountingMonoid<Xsk233MulHashMonoid>>::lift(&item);

        let count = <_ as ProtocolMonoid>::count(&monoid);
        assert_eq!(1, count);
    }
}
