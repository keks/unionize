pub mod uniform {
    use crate::{
        item::le_byte_array::LEByteArray,
        monoid::{count::CountingMonoid, mulhash_xs233::MulHashMonoid},
        tree::mem_rc::Node as MemRcNode,
    };

    use xs233::xsk233::Xsk233Point;

    pub type Item = LEByteArray<30>;
    pub type Monoid = CountingMonoid<MulHashMonoid<Xsk233Point>>;
    pub type Node = MemRcNode<Monoid>;
}
