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

        'L:
        for (range, part) in parts {
            match part {
                MessagePart::Fingerprint(fp) => {
                    let root: &Node<M> = &root;
                    if root.monoid() == fp {
                        break 'L;
                    }

                    // we need to split now.
                    let (left, right) = match range {
                        Range::Full => (1,2),
                        Range::UpTo(_) => todo!(),
                        Range::StartingFrom(_) => todo!(),
                        Range::Between(_, _) => todo!(),
                    };
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