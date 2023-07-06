use crate::{
    monoid::{Item, Monoid},
    Node,
};

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

    pub(crate) fn has_overlap<'a, N, M>(&self, node: &'a N) -> bool
    where
        M: Monoid<Item = T> + 'a,
        N: Node<'a, M>,
    {
        if node.is_nil() {
            return false;
        }

        let min = node
            .min_item()
            .expect("can only fail with nil node, node isn't nil");
        let max = node
            .max_item()
            .expect("can only fail with nil node, node isn't nil");

        max >= self.from() || min < self.to()
    }

    pub(crate) fn fully_contains<'a, M, N>(&self, node: &'a N) -> bool
    where
        M: Monoid<Item = T> + 'a,
        N: Node<'a, M>,
    {
        if node.is_nil() {
            return false;
        }

        let min = node
            .min_item()
            .expect("can only fail with nil node, node isn't nil");
        let max = node
            .max_item()
            .expect("can only fail with nil node, node isn't nil");

        if self.is_full() {
            true
        } else if self.is_wrapping() {
            min >= self.from() || max < self.to()
        } else {
            min >= self.from() && max < self.to()
        }
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
                (_, std::cmp::Ordering::Equal) => RangeCompare::IsUpperBound,
                (std::cmp::Ordering::Equal, _) => RangeCompare::IsLowerBound,

                (_, std::cmp::Ordering::Less) | (std::cmp::Ordering::Greater, _) => {
                    RangeCompare::Included
                }

                (std::cmp::Ordering::Less, std::cmp::Ordering::Greater) => RangeCompare::InBetween,
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
            RangeCompare::GreaterThan | RangeCompare::InBetween => Some(self.clone()),
            RangeCompare::IsLowerBound | RangeCompare::LessThan => None,
        }
    }

    pub(crate) fn cap_left(&self, new_start: T) -> Option<Self> {
        match self.cmp(&new_start) {
            RangeCompare::IsLowerBound | RangeCompare::Included => {
                Some(Self(new_start, self.1.clone()))
            }
            RangeCompare::LessThan | RangeCompare::InBetween => Some(self.clone()),
            RangeCompare::IsUpperBound | RangeCompare::GreaterThan => None,
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
