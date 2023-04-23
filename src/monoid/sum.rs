use crate::LiftingMonoid;

use super::FormattingMonoid;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SumMonoid(pub u64);

impl LiftingMonoid for SumMonoid {
    type Item = u64;

    fn neutral() -> Self {
        SumMonoid(0)
    }

    fn lift(item: &Self::Item) -> Self {
        SumMonoid(*item)
    }

    fn combine(&self, other: &Self) -> Self {
        let (SumMonoid(lhs), SumMonoid(rhs)) = (self, other);
        SumMonoid(*lhs + *rhs)
    }
}


impl FormattingMonoid for SumMonoid {
    fn item_to_string(item: &Self::Item) -> String {
        format!("{item}")
    }
}
