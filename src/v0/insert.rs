use super::{Cursor, Key, Monoid, Node};

struct Insert<K: Key, M: Monoid> {
    cursor: Cursor<K, M>,
    new_elem: K,
}
