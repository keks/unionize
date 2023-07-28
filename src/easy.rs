/// Good choices for types when aiming for uniformly distributed items and aiming for
/// collision-resistance.
pub mod uniform {
    use crate::{
        item::le_byte_array::LEByteArray,
        monoid::{count::CountingMonoid, mulhash_xs233::Xsk233MulHashMonoid},
        tree::mem_rc::Node as MemRcNode,
    };

    extern crate alloc;
    use alloc::{vec, vec::Vec};

    pub type Item = LEByteArray<30>;
    pub type Monoid = CountingMonoid<Xsk233MulHashMonoid>;
    pub type Node = MemRcNode<Monoid>;

    pub fn split<const C: usize>(n: usize) -> Vec<usize> {
        let most = n / C;
        let rest = n - (C - 1) * most;

        let mut out = vec![most; C];
        out[0] = rest;
        out
    }
}

pub mod timestamped {
    extern crate alloc;
    use alloc::vec::Vec;

    pub fn split<const C: usize>(n: usize) -> Vec<usize> {
        let mut res = Vec::with_capacity(C);
        let mut cur = n;
        for _ in 1..C {
            let half = cur / 2;
            res.push(cur - half);
            cur = half;
        }
        res.push(cur);

        res
    }

    pub fn split_dynamic<const THRESH: usize>(n: usize) -> Vec<usize> {
        let mut res = Vec::with_capacity(n.ilog2() as usize + 3);
        let mut cur = n;
        while cur >= THRESH {
            let half = cur / 2;
            res.push(cur - half);
            cur = half;
        }
        res.push(cur);

        res
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use crate::{
        monoid::{count::CountingMonoid, sum::SumMonoid},
        tree::mem_rc::Node,
    };

    // helper type for tests
    pub type TestObject = (u64, bool);
    pub type TestItem = u64;
    pub type TestMonoid = CountingMonoid<SumMonoid<TestItem>>;
    pub type TestNode = Node<TestMonoid>;
}
