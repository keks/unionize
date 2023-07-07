macro_rules! impl_Item_uint {
    ($type:ty) => {
        impl $crate::item::Item for $type {
            fn zero() -> Self {
                0
            }

            fn next(&self) -> Self {
                self + 1
            }
        }
    };
}

impl_Item_uint!(u8);
impl_Item_uint!(u16);
impl_Item_uint!(u32);
impl_Item_uint!(u64);
impl_Item_uint!(u128);
