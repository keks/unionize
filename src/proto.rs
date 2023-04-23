use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    rc::Rc,
};

use crate::{
    monoid::{count::CountingMonoid, FormattingMonoid},
    range::Range,
    tree::{cursor::Cursor, NodeData},
    LiftingMonoid, Node,
};

pub(crate) trait ProtocolMonoid: FormattingMonoid {
    fn count(&self) -> usize;
}

enum MessagePart<M: ProtocolMonoid> {
    Fingerprint(M),
    ItemSet(Vec<M::Item>, bool),
}

impl<M: ProtocolMonoid> MessagePart<M> {
    fn fingerprint(fp: M) -> Self {
        Self::Fingerprint(fp)
    }

    fn item_set(items: Vec<M::Item>, already_received: bool) -> Self {
        Self::ItemSet(items, already_received)
    }

    /// Returns `true` if the message part is [`Fingerprint`].
    ///
    /// [`Fingerprint`]: MessagePart::Fingerprint
    #[must_use]
    fn is_fingerprint(&self) -> bool {
        matches!(self, Self::Fingerprint(..))
    }
}

struct Message<M: ProtocolMonoid>(Vec<(Range<M::Item>, MessagePart<M>)>);


fn respond_to_message<M: ProtocolMonoid>(
    root: Rc<Node<M>>,
    msg: &Message<M>,
) -> (Message<M>, Vec<M::Item>) {
    (Message(vec![]), vec![])   
}