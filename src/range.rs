use std::fmt::Debug;

pub trait Rangable: std::fmt::Debug + Ord + Clone {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Range<T: Rangable>(pub(crate) T, pub(crate) T);

impl<T: Rangable> Range<T> {
    pub fn reverse(&self) -> Self {
        let Self(from, to) = self;
        Self(to.clone(), from.clone())
    }

    pub fn from(&self) -> &T {
        &self.0
    }
    pub fn to(&self) -> &T {
        &self.1
    }

    pub(crate) fn is_wrapping(&self) -> bool {
        let Range(from, to) = self;
        from >= to
    }

    pub(crate) fn is_full(&self) -> bool {
        let Range(from, to) = self;
        from == to
    }

    pub(crate) fn contains(&self, item: &T) -> bool {
        if self.is_full() {
            return true;
        }

        let Range(from, to) = self;
        if self.is_wrapping() {
            from <= item || item < to
        } else {
            from <= item && item < to
        }
    }

    pub(crate) fn cmp(&self, item: &T) -> RangeCompare {
        let Range(from, to) = self;
        if self.is_full() {
            RangeCompare::Included
        } else if self.is_wrapping() {
            match (from.cmp(item), to.cmp(item)) {
                (_, std::cmp::Ordering::Less) | (std::cmp::Ordering::Greater, _) => {
                    RangeCompare::Included
                }
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => RangeCompare::InBetween,
            }
        } else {
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

        Self(new_start, self.1.clone())
    }
}

impl<T: Rangable + Copy> Copy for Range<T> {}

impl<T: Rangable + std::fmt::Debug> std::fmt::Display for Range<T> {
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

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn wat() {
        let range = Range(228, 1);
        assert!(range.is_wrapping(), "should be wrapping");
        assert!(!range.is_full(), "shouldn't be full");

        let item = 227;
        assert!(!range.contains(&item), "shouldn't contain item")
    }
}
