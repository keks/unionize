use crate::hash_item::LEByteArray;

use super::{Item, Monoid, Peano};

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub struct MulHashMonoid<P: xs233::Point>(P);

impl<const L: usize> Item for [u8; L] {}

impl<const L: usize> Peano for [u8; L] {
    fn zero() -> Self {
        [0u8; L]
    }

    fn next(&self) -> Self {
        let mut result: [u8; L] = self.clone();
        for i in 0..L {
            let (sum, did_overflow) = result[i].overflowing_add(1);
            result[i] = sum;
            if !did_overflow {
                break;
            }
        }
        result
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

#[derive(Debug, Copy, Clone)]
pub struct InvalidPoint;

#[derive(Clone, PartialEq, Eq)]
pub struct EncodedPoint<const L: usize>(pub [u8; L]);

impl<const L: usize> ::core::fmt::Debug for EncodedPoint<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl<const L: usize, P> crate::proto::Encodable for MulHashMonoid<P>
where
    P: xs233::Point<EncodedPoint = [u8; L]> + Eq + 'static,
{
    type Encoded = EncodedPoint<L>;
    type Error = InvalidPoint;

    fn encode(&self, target: &mut Self::Encoded) -> Result<(), Self::Error> {
        P::encode(&self.0, &mut target.0);
        Ok(())
    }

    fn decode(&mut self, target: &Self::Encoded) -> Result<(), Self::Error> {
        if P::decode(&mut self.0, &target.0).into() {
            Ok(())
        } else {
            Err(InvalidPoint)
        }
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
    use super::*;

    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn xsk_lift_and_add_1m_items() {
        let mut buf = [0u8; 30];
        let mut rng = ChaCha8Rng::from_seed([23u8; 32]);
        let mut acc = MulHashMonoid::<xs233::xsk233::Xsk233Point>::neutral();

        for _ in 0..1_000_000 {
            rng.fill(&mut buf);
            let pt = MulHashMonoid::lift(&LEByteArray(buf));
            acc = acc.combine(&pt)
        }

        println!("{acc:?}");
    }
}
