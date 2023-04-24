use std::rc::Rc;

use crate::{
    monoid::FormattingMonoid,
    range::{Rangable, Range},
    Node,
};

pub trait ProtocolMonoid: FormattingMonoid {
    fn count(&self) -> usize;
}

pub enum MessagePart<M: ProtocolMonoid> {
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

pub struct Message<M: ProtocolMonoid>(Vec<(Range<M::Item>, MessagePart<M>)>)
where
    M::Item: Rangable;

pub fn respond_to_message<M: ProtocolMonoid>(
    root: Rc<Node<M>>,
    msg: &Message<M>,
) -> (Message<M>, Vec<M::Item>)
where
    M::Item: Rangable,
{
    (Message(vec![]), vec![])
}
