extern crate alloc;
extern crate std;

use core::convert::Infallible;

use crate::{
    item::le_byte_array::LEByteArray,
    protocol::{encoding::AsDestMutRef, DecodeError, EncodeError},
};

use alloc::format;
use serde::{de::Deserializer, Deserialize, Serialize};

use super::Monoid;

pub type Xsk233MulHashMonoid = MulHashMonoid<xs233::xsk233::Xsk233Point>;

/// MulHashMonoid lifts values by mapping them to points on an elliptic curve using a
/// decoding-rejection-sampling technique (i.e. we try to decode and if that fails try again with a
/// deterministically changed item).
/// Combining works by adding the curve points.
/// This should be cryptographically secure. I hope.
#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct MulHashMonoid<P: xs233::Point>(P);

impl<P: xs233::Point> MulHashMonoid<P> {
    #[allow(dead_code)] // is actually used, but in tests
    pub(crate) fn set(&mut self, pt: P) {
        self.0 = pt;
    }
}

impl<const L: usize, P: xs233::Point<EncodedPoint = [u8; L]> + Eq + 'static> Monoid
    for MulHashMonoid<P>
{
    type Item = LEByteArray<L>;

    fn neutral() -> Self {
        Self(P::neutral().clone())
    }

    fn lift(item: &Self::Item) -> Self {
        Self(xs233::map_uniform_bytes_to_curve(item.0.clone()))
    }

    fn combine(&self, other: &Self) -> Self {
        let mut out = P::default();
        out.add(&self.0, &other.0);
        Self(out)
    }
}

impl<const L: usize> Serialize for EncodedPoint<L> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

struct EncodedPointVisitor<const L: usize>([(); L]);

impl<'de, const L: usize> serde::de::Visitor<'de> for EncodedPointVisitor<L> {
    type Value = EncodedPoint<L>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str(&format!("{L} bytes/u8s"))
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value.try_into() {
            Ok(buf) => Ok(EncodedPoint(buf)),
            Err(_) => Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Bytes(value),
                &self,
            )),
        }
    }
}

impl<'de, const L: usize> Deserialize<'de> for EncodedPoint<L> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(EncodedPointVisitor([(); L]))
    }
}

/// This error is returned when point decoding fails.
#[derive(Debug, Copy, Clone)]
pub struct InvalidPoint;

impl core::fmt::Display for InvalidPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("invalid point")
    }
}

impl std::error::Error for InvalidPoint {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

/// Represents an encoded point. This representation is far more compact than MulHashMonoid, but
/// can't be used for computation. Good for serialization.
#[derive(Clone, PartialEq, Eq)]
pub struct EncodedPoint<const L: usize>(pub [u8; L]);

impl<const L: usize> ::core::fmt::Debug for EncodedPoint<L> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut short = [0u8; 4];
        for i in 0..4 {
            short[i] = self.0[i];
        }

        let hex_str = hex::encode(&short);

        write!(f, "EncPt({hex_str})")
    }
}

impl<const L: usize> Default for EncodedPoint<L> {
    fn default() -> Self {
        EncodedPoint([0u8; L])
    }
}

impl<const L: usize, P> crate::protocol::Encodable for MulHashMonoid<P>
where
    P: xs233::Point<EncodedPoint = [u8; L]> + Eq + 'static,
{
    type Encoded = EncodedPoint<L>;
    type EncodeError = Infallible;
    type DecodeError = InvalidPoint;

    fn encode(&self, target: &mut Self::Encoded) -> Result<(), EncodeError<Self::EncodeError>> {
        P::encode(&self.0, &mut target.0);
        Ok(())
    }

    fn decode(&mut self, target: &Self::Encoded) -> Result<(), DecodeError<Self::DecodeError>> {
        if P::decode(&mut self.0, &target.0).into() {
            Ok(())
        } else {
            Err(DecodeError(InvalidPoint))
        }
    }

    fn batch_encode<Dst: AsDestMutRef<Self::Encoded>>(
        src: &[Self],
        dst: &mut [Dst],
    ) -> Result<(), EncodeError<Self::EncodeError>> {
        assert_eq!(src.len(), dst.len());

        for i in 0..src.len() {
            P::encode(&src[i].0, &mut dst[i].as_dest_mut_ref().0);
        }

        Ok(())
    }
}

/*
 * Implementation notes:
 * - I think I need to encode and decode in between. it seems like this is a lot more expensive
 *   doing the actual adding. maybe find a way to batch this?
 *   - how is the real world use? how much batching can I even do? is it worth the complexity, or
 *     should I just encode and decode each time?
 *   - actually performance is still not tooo bad. <1s in balanced, <3s in powersave, for 100k
 *     lift+add.
 *   - a good way to handle this would be to add a batch combine function that has a defualt
 *     naive implementation that can be overridden with something more efficient.
 *
 * */

#[cfg(test)]
mod tests {
    extern crate std;
    use std::println;

    use super::*;

    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn xsk_lift_and_add_100k_items() {
        let mut buf = [0u8; 30];
        let mut rng = ChaCha8Rng::from_seed([23u8; 32]);
        let mut acc = MulHashMonoid::<xs233::xsk233::Xsk233Point>::neutral();

        for _ in 0..100_000 {
            rng.fill(&mut buf);
            let pt = MulHashMonoid::lift(&LEByteArray(buf));
            acc = acc.combine(&pt)
        }

        println!("{acc:?}");
    }

    use proptest::{prop_assert_eq, proptest};

    proptest! {
        #[test]
        fn serialize_correctness(data in proptest::array::uniform30(0u8..=255u8)) {
            println!("d:{data:x?}");
            let point = EncodedPoint(data);
            let encoded = serde_cbor::to_vec(&point).unwrap();
            println!("e:{encoded:x?}");
            let result = serde_cbor::from_slice(&encoded).unwrap();
            prop_assert_eq!(point, result);
        }
    }
}
