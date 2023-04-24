use std::fmt::Debug;

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
            return true;
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
                (_, std::cmp::Ordering::Less) | (std::cmp::Ordering::Greater, _) => {
                    RangeCompare::Included
                }
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

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

                (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => RangeCompare::LessThan,
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                    RangeCompare::GreaterThan
                }
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
                Ord::max(&self.1, &other.1).clone(),
            ))
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
                Ord::max(&self.1, &other.1).clone(),
            ))
        }
    }

    pub(crate) fn has_overlap(&self, other: &Self) -> bool {
        if self.is_full() || other.is_full() {
            return true;
        }

        match (self.is_wrapping(), other.is_wrapping()) {
            (true, true) => true, // they at least overlap at the wrap
            (false, false) => !(self.from() >= other.to() || other.from() >= self.to()),

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
                Ord::min(&self.1, &other.1).clone(),
            ))
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
                    Ord::max(&self.1, &other.1).clone(),
                ))
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
        assert!(matches!(
            self.cmp(&new_end),
            RangeCompare::Included | RangeCompare::IsUpperBound
        ));
        assert!(!self.is_wrapping() || self.is_full());

        Self(self.0.clone(), new_end)
    }

    pub(crate) fn cap_left(&self, new_start: T) -> Self {
        assert!(matches!(
            self.cmp(&new_start),
            RangeCompare::Included | RangeCompare::IsLowerBound
        ));
        assert!(!self.is_wrapping() || self.is_full());

        Self(self.0.clone(), new_start)
    }
}

impl<T: Rangable + std::fmt::Debug> std::fmt::Display for NewRange<T> {
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
    InBetween,
}
