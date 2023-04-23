use crate::{tree::ChildId, LiftingMonoid};

use super::Node;

use std::fmt::Debug;

// I know I'm reinventing ranges but they are not enum.
// Maybe I can make this implement From<Range*>
#[derive(Debug, Clone)]
pub(crate) enum Range<T: std::fmt::Debug + Ord + Clone> {
    Full,
    UpTo(T),
    StartingFrom(T),
    Between(T, T),
}

pub trait Rangable: std::fmt::Debug + Ord + Clone {}

#[derive(Debug, Clone)]
pub struct NewRange<T: Rangable>(pub(crate) T, pub(crate) T);

impl<T: Rangable> NewRange<T> {
    pub fn from(&self) -> &T {
        if self.is_wrapping() {
            &self.1
        } else {
            &self.0
        }
    }
    pub fn to(&self) -> &T {
        if self.is_wrapping() {
            &self.0
        } else {
            &self.1
        }
    }

    pub(crate) fn is_wrapping(&self) -> bool {
        let NewRange(from, to) = self;
        from >= to
    }

    pub(crate) fn is_full(&self) -> bool {
        let NewRange(from, to) = self;
        from == to
    }

    pub(crate) fn contains(&self, item: &T) -> bool {
        let NewRange(from, to) = self;
        if from == to {
            return true
        }

        if self.is_wrapping() {
            let (from, to) = (to, from);
            from <= item || item < to
        } else {
            from <= item && item < to
        }
    }

    pub(crate) fn cmp(&self, item: &T) -> RangeCompare {
        if self.is_full() {
            RangeCompare::Included
        } else if self.is_wrapping() {
            let NewRange(to, from) = self;
            match (from.cmp(item), to.cmp(item)) {
                (_, std::cmp::Ordering::Less) |
                (std::cmp::Ordering::Greater, _) => RangeCompare::Included,
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                // actually we can't tell if it's greater than or less than because it's cyclic, but we'll just use gt here
                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => RangeCompare::InBetween,
            }
        } else {
            let NewRange(from, to) = self;
            match (from.cmp(item), to.cmp(item)) {
                // this can only occur for wrapping ranges
                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => unreachable!(),

                (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => RangeCompare::Included,
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                (std::cmp::Ordering::Less, std::cmp::Ordering::Less)  => RangeCompare::LessThan,
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => RangeCompare::GreaterThan,
            }
        }
    }

    pub(crate) fn is_consecutive(&self, other: &Self) -> bool {
        if self.is_wrapping() && other.is_wrapping() {
            false
        } else if self.is_wrapping() || other.is_wrapping() {
            self.0 == other.0 || self.1 == other.1
        } else {
            self.0 == other.1 || self.1 == other.0
        }
    }

    pub(crate) fn union(&self, other: &Self) -> Option<Self> {
        if !self.has_overlap(other) {
            None
        } else if self.is_wrapping() && other.is_wrapping() {
            Some(Self(
                Ord::min(&self.0, &other.0).clone(),
                Ord::max(&self.1, &other.1).clone()))
        } else if self.is_wrapping() || other.is_wrapping() {
            if self.0 <= other.1 {
                Some(Self(other.0.clone(), self.1.clone()))
            } else if self.1 <= other.0 {
                Some(Self(self.0.clone(), other.1.clone()))
            } else {
                unreachable!()
            }
        } else {
            Some(Self(
                Ord::min(&self.0, &other.0).clone(), 
                Ord::max(&self.1, &other.1).clone()))
        }
    }

    pub(crate) fn has_overlap(&self, other: &Self) -> bool {
        if self.is_full() || other.is_full() {
            return true
        }

        match (self.is_wrapping(), other.is_wrapping()) {
            (true, true) => true, // they at least overlap at the wrap
            (false, false) => {
                !(self.from() >= other.to() || other.from() >= self.to())
            }

            _ => self.from() < other.to() || other.from() < self.to(),
        }

    }

    pub(crate) fn intersect(&self, other: &Self) -> Option<NewRange<T>> {
        if self.is_full() {
            Some(other.clone())
        } else if other.is_full() {
            Some(self.clone())
        } else if self.is_wrapping() && other.is_wrapping() {
            Some(Self(
                Ord::max(&self.0, &other.0).clone(),
                Ord::min(&self.1, &other.1).clone()))
        } else if self.is_wrapping() || other.is_wrapping() {
            // NOTE: when exactly one of the two is wrapping, we can have the situation that there is overlap in two places. In that case we only return the lower range
            if self.1 > other.0 {
                Some(Self(other.0.clone(), self.1.clone()))
            } else if self.0 < other.1 {
                Some(Self(self.0.clone(), other.1.clone()))
            } else {
                None // no intersection, empty set (having both items the same means full set)
            }
        } else {
            if self.has_overlap(other) {
                Some(Self(
                    Ord::min(&self.0, &other.0).clone(),
                    Ord::max(&self.1, &other.1).clone()))
            } else {
                None
            }
        }
    }

    pub(crate) fn is_subrange_of(&self, other: &Self) -> bool {
        if other.is_full() {
            true
        } else if self.is_full() {
            false
        } else if self.is_wrapping() && other.is_wrapping() {
            self.0 >= other.1 && self.1 <= other.1
        } else if self.is_wrapping() {
            false
        } else if other.is_wrapping() {
            self.0 >= other.0 || self.1 <= other.1
        } else {
            self.0 >= other.0 && self.1 <= other.1
        }
    }

    pub(crate) fn cap_right(&self, new_end: T) -> Self {
        assert!(matches!(self.cmp(&new_end), RangeCompare::Included | RangeCompare::IsUpperBound));
        assert!(!self.is_wrapping() || self.is_full());

        Self(self.0.clone(), new_end)
    }


    pub(crate) fn cap_left(&self, new_start: T) -> Self {
        assert!(matches!(self.cmp(&new_start), RangeCompare::Included | RangeCompare::IsLowerBound));
        assert!(!self.is_wrapping() || self.is_full());

        Self(self.0.clone(), new_start)
    }
}


impl<T: std::fmt::Debug + Ord + Clone> std::fmt::Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::Full => write!(f, ".."),
            Range::UpTo(x) => write!(f, "..{x:?}"),
            Range::StartingFrom(x) => write!(f, "{x:?}.."),
            Range::Between(x, y) => write!(f, "{x:?}..{y:?}"),
        }
    }
}

impl<T: Rangable + std::fmt::Debug > std::fmt::Display for NewRange<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(from, to) = self;
        write!(f, "{from:?}..{to:?}")
    }
}

#[derive(Debug)]
pub(crate) enum RangeCompare {
    LessThan,
    IsLowerBound,
    Included,
    GreaterThan,
    IsUpperBound,
    InBetween, // used only for newrange comparisons
}

impl<T: Debug + Ord + Clone> Range<T> {
    pub fn contains(&self, item: &T) -> bool {
        match self {
            Self::Full => true,
            Self::UpTo(x) => item < x,
            Self::StartingFrom(x) => item >= x,
            Self::Between(x, y) => item >= x && item < y,
        }
    }

    pub(crate) fn cmp(&self, item: &T) -> RangeCompare {
        match self {
            Self::Full => RangeCompare::Included,
            Self::UpTo(x) if item < x => RangeCompare::Included,
            Self::UpTo(x) if item == x => RangeCompare::IsUpperBound,
            Self::UpTo(_) => RangeCompare::GreaterThan,
            Self::StartingFrom(x) if item > x => RangeCompare::Included,
            Self::StartingFrom(x) if item == x => RangeCompare::IsLowerBound,
            Self::StartingFrom(_) => RangeCompare::LessThan,
            Self::Between(x, _) if item < x => RangeCompare::LessThan,
            Self::Between(_, y) if item == y => RangeCompare::IsUpperBound, // this needs to be before the ==x check
            Self::Between(x, _) if item == x => RangeCompare::IsLowerBound,
            Self::Between(_, y) if item > y => RangeCompare::GreaterThan,
            Self::Between(_, _) => RangeCompare::Included,
        }
    }

    pub fn with_end(&self, item: T) -> Self {
        match self {
            Self::Full => Self::UpTo(item),
            Self::StartingFrom(x) => {
                assert!(x < &item, "making a range with start < end");
                Self::Between(x.clone(), item)
            }
            Self::UpTo(x) => {
                assert!(x > &item, "new end {item:?} is larger than old end {x:?}");
                Self::UpTo(item)
            }
            Self::Between(x, y) => {
                assert!(x < &item, "making a range with start < end");
                assert!(y > &item, "new end is larger than old end");
                Self::Between(x.clone(), item)
            }
        }
    }

    pub fn with_start(&self, item: T) -> Self {
        match self {
            Self::Full => Self::StartingFrom(item),
            Self::StartingFrom(x) => {
                assert!(&item > x, "new start is less than old start");
                Self::StartingFrom(item)
            }
            Self::UpTo(x) => {
                assert!(&item < x, "making a range with start < end");
                Self::Between(item, x.clone())
            }
            Self::Between(x, y) => {
                assert!(&item < y, "making a range with start < end");
                assert!(&item > x, "new start is less than old start");
                Self::Between(item, y.clone())
            }
        }
    }

    pub fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (Range::UpTo(x), Range::UpTo(y)) => Range::UpTo(T::min(x.clone(), y.clone())),
            (Range::StartingFrom(x), Range::StartingFrom(y)) => {
                Range::StartingFrom(T::max(x.clone(), y.clone()))
            }
            (Range::Between(x1, y1), Range::Between(x2, y2)) => {
                let x = T::max(x1.clone(), x2.clone());
                let y = T::min(y1.clone(), y2.clone());
                assert!(x <= y);
                Range::Between(x, y)
            }

            (Range::Full, x) | (x, Range::Full) => x.clone(),

            (Range::StartingFrom(x), Range::UpTo(y)) | (Range::UpTo(y), Range::StartingFrom(x)) => {
                assert!(x <= y);
                Range::Between(x.clone(), y.clone())
            }

            (Range::Between(x, y1), Range::UpTo(y2)) | (Range::UpTo(y2), Range::Between(x, y1)) => {
                let y = T::min(y1.clone(), y2.clone());
                assert!(x <= &y);
                Range::Between(x.clone(), y)
            }

            (Range::Between(x1, y), Range::StartingFrom(x2))
            | (Range::StartingFrom(x2), Range::Between(x1, y)) => {
                let x = T::max(x1.clone(), x2.clone());
                assert!(&x <= y);
                Range::Between(x, y.clone())
            }
        }
    }

    pub fn is_subrange_of(&self, other: &Self) -> bool {
        match (self, other) {
            (_, Range::Full) => true,
            (Range::Full, _) => false,

            (Range::UpTo(x), Range::UpTo(y)) => x <= y,
            (Range::StartingFrom(x), Range::StartingFrom(y)) => x >= y,
            (Range::Between(x1, y1), Range::Between(x2, y2)) => x1 >= x2 && y1 <= y2,

            _ => false,
        }
    }

    pub fn has_overlap(&self, other: &Self) -> bool {
        match (self, other) {
            (Range::Full, _) | (_, Range::Full) => true,


            // ?-----
            //   ?---
            (Range::UpTo(_), Range::UpTo(_)) |
            // ---?
            // -----?
            (Range::StartingFrom(_), Range::StartingFrom(_)) => true,

            // x---..
            // ..---y
            (Range::StartingFrom(x), Range::UpTo(y)) |
            (Range::UpTo(y), Range::StartingFrom(x)) |
            
            // x---?
            // x-----?
            // ..---y
            (Range::UpTo(y), Range::Between(x, _)) |
            (Range::Between(x, _), Range::UpTo(y)) |

            // ?-----y
            //   ?---y
            //  x---..
            (Range::StartingFrom(x), Range::Between(_, y)) |
            (Range::Between(_, y), Range::StartingFrom(x)) => x <= y,

            //  x1---y1
            //    x2---?
            // ?---y2
            (Range::Between(x1, y1), Range::Between(x2, y2)) => (x1 >= x2 && x1 <= y2) || ( y2 >= x1 && y2 <= y2 ),
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Between(x, y) => x <= y,
            _ => true,
        }
    }
}
