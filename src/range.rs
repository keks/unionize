use crate::monoid::Item;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Range<T: Item>(pub(crate) T, pub(crate) T);

impl<T: Item + Copy> Copy for Range<T> {}

impl<T: Item> std::fmt::Display for Range<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(from, to) = self;
        write!(f, "{from:?}..{to:?}")
    }
}

impl<T: Item> Range<T> {
    pub fn reverse(&self) -> Self {
        let Self(from, to) = self;
        Self(to.clone(), from.clone())
    }

    #[inline]
    pub fn from(&self) -> &T {
        &self.0
    }

    #[inline]
    pub fn to(&self) -> &T {
        &self.1
    }

    #[inline]
    pub(crate) fn is_wrapping(&self) -> bool {
        let Range(from, to) = self;
        from >= to
    }

    #[inline]
    pub(crate) fn is_full(&self) -> bool {
        let Range(from, to) = self;
        from == to
    }

    #[inline]
    pub(crate) fn contains(&self, item: &T) -> bool {
        let Range(from, to) = self;
        if self.is_wrapping() {
            from <= item || item < to
        } else {
            from <= item && item < to
        }
    }

    #[inline]
    pub(crate) fn partially_contains(&self, min: &T, max: &T) -> bool {
        min < self.to() || max >= self.from()
    }

    #[inline]
    pub(crate) fn fully_contains(&self, min: &T, max: &T) -> bool {
        let Range(from, to) = self;
        if self.is_full() {
            true
        } else if self.is_wrapping() {
            from <= min || max < to
        } else {
            from <= min && max < to
        }
    }

    pub(crate) fn cmp(&self, item: &T) -> RangeCompare {
        let Range(from, to) = self;
        if self.is_full() {
            RangeCompare::Included
        } else if self.is_wrapping() {
            match (from.cmp(item), to.cmp(item)) {
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                (std::cmp::Ordering::Less, _) | (_, std::cmp::Ordering::Greater) => {
                    RangeCompare::Included
                }

                (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => RangeCompare::InBetween,
            }
        } else {
            match (from.cmp(item), to.cmp(item)) {
                // this can only occur for wrapping ranges
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Less) => {
                    unreachable!("can only happen for wrapping ranges, and this query doesn't wrap")
                }

                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => RangeCompare::Included,

                (std::cmp::Ordering::Less, std::cmp::Ordering::Less) => RangeCompare::LessThan,
                (std::cmp::Ordering::Greater, std::cmp::Ordering::Greater) => {
                    RangeCompare::GreaterThan
                }
            }
        }
    }

    pub(crate) fn cap_right(&self, new_end: T) -> Option<Self> {
        match self.cmp(&new_end) {
            RangeCompare::IsUpperBound | RangeCompare::Included => {
                Some(Self(self.0.clone(), new_end))
            }
            RangeCompare::GreaterThan => Some(self.clone()),
            RangeCompare::IsLowerBound | RangeCompare::LessThan | RangeCompare::InBetween => None,
        }
    }

    pub(crate) fn cap_left(&self, new_start: T) -> Option<Self> {
        let cmp = self.cmp(&new_start);
        println!("cap_left range:{self:?} new_start:{new_start:?} cmp:{cmp:?}");
        match cmp {
            RangeCompare::IsLowerBound | RangeCompare::Included => {
                Some(Self(new_start, self.1.clone()))
            }
            RangeCompare::LessThan => Some(self.clone()),
            RangeCompare::IsUpperBound | RangeCompare::GreaterThan | RangeCompare::InBetween => {
                None
            }
        }
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
