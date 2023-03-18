use std::rc::Rc;

use crate::{LiftingMonoid, range::Range, Node, sexpr::SiseMonoid};

enum MessagePart<M: LiftingMonoid> {
    Fingerprint(M),
    ItemSet(Vec<M::Item>, bool),
}

struct Message<M: LiftingMonoid>(Vec<(Range<M::Item>, MessagePart<M>)>);

impl<M: LiftingMonoid + SiseMonoid> Message<M> {
    fn respond(&self, root: Rc<Node<M>>) -> (Self, Vec<M::Item>) {
        let Message(parts) = self;

        let mut resp_parts = vec![];
        let mut new_items = vec![];

        for (range, part) in parts {
            match part {
                MessagePart::Fingerprint(fp) => {

                }
                MessagePart::ItemSet(items, already_received) => {
                    new_items.extend(items.iter().cloned());
                    let own_items = Node::items(Rc::clone(&root), range.clone()).collect();

                    if !already_received {
                        resp_parts.push((range.clone(), MessagePart::ItemSet(own_items, true)))
                    }
                }
            }
        }

        (Self(resp_parts), new_items)
    }
}