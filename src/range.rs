use crate::item::Item;

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
        let wrapping_case = from <= item || item < to;
        let non_wrapping_case = from <= item && item < to;
        let is_wrapping = self.is_wrapping();

        (is_wrapping && wrapping_case) || non_wrapping_case
        // equivalent to:
        // if self.is_wrapping() {
        //     wrapping_case
        // } else {
        //     non_wrapping_case
        // }
    }

    #[inline]
    pub(crate) fn partially_contains(&self, min: &T, max: &T) -> bool {
        let node_bounds_around_query_range = min < self.from() && self.to() <= max;

        self.contains(min) || self.contains(max) || node_bounds_around_query_range
    }

    #[inline]
    pub(crate) fn fully_contains(&self, min: &T, max: &T) -> bool {
        let Range(from, to) = self;
        let is_full = self.is_full();
        let is_wrapping = self.is_wrapping();

        let wrapping_case = from <= min || max < to;
        let non_wrapping_case = from <= min && max < to;

        is_full || (is_wrapping && wrapping_case) || non_wrapping_case
        // equivalent to:
        // if self.is_full() {
        //     true
        // } else if self.is_wrapping() {
        //     wrapping_case
        // } else {
        //     non_wrapping_case
        // }
    }
}
