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
