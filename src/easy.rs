/// Good choices for types when aiming for uniformly distributed items and aiming for
/// collision-resistance.
pub mod uniform {
    use crate::{
        item::le_byte_array::LEByteArray,
        monoid::{count::CountingMonoid, mulhash_xs233::MulHashMonoid},
        tree::mem_rc::Node as MemRcNode,
    };

    extern crate alloc;
    use alloc::{vec, vec::Vec};

    use xs233::xsk233::Xsk233Point;

    pub type Item = LEByteArray<30>;
    pub type Monoid = CountingMonoid<MulHashMonoid<Xsk233Point>>;
    pub type Node = MemRcNode<Monoid>;

    pub fn split<const C: usize>(n: usize) -> Vec<usize> {
        let most = n / C;
        let rest = n - (C - 1) * most;

        let mut out = vec![most; C];
        out[0] = rest;
        out
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        monoid::{count::CountingMonoid, sum::SumMonoid},
        tree::mem_rc::Node,
    };

    // helper type for tests
    pub type TestItem = u64;
    pub type TestMonoid = CountingMonoid<SumMonoid<TestItem>>;
    pub type TestNode = Node<TestMonoid>;
}
