use super::Item;

impl Item for u32 {
    fn zero() -> Self {
        0
    }

    fn next(&self) -> Self {
        self + 1
    }
}

impl Item for u64 {
    fn zero() -> Self {
        0
    }

    fn next(&self) -> Self {
        self + 1
    }
}
