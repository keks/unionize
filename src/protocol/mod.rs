extern crate alloc;
use alloc::{vec, vec::Vec};

extern crate std;

pub mod encoding;
pub use encoding::{DecodeError, Encodable, EncodeError};

pub mod error;
pub use error::RespondError;

use crate::{
    item::Item,
    monoid::Monoid,
    query::{items::ItemsAccumulator, simple::SimpleAccumulator, split::SplitAccumulator},
    range::Range,
    Node, NonNilNodeRef,
};

pub trait ProtocolMonoid: Monoid + Encodable {
    fn count(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct FingerprintRecord<M: ProtocolMonoid> {
    range: Range<M::Item>,
    fp: M::Encoded,
}

impl<M: ProtocolMonoid> FingerprintRecord<M> {
    pub fn new(range: Range<M::Item>, fp: M::Encoded) -> Self {
        Self { range, fp }
    }

    pub fn range(&self) -> &Range<M::Item> {
        &self.range
    }

    pub fn fp(&self) -> &M::Encoded {
        &self.fp
    }
}

impl<M: ProtocolMonoid> encoding::AsDestMutRef<M::Encoded> for FingerprintRecord<M> {
    fn as_dest_mut_ref(&mut self) -> &mut M::Encoded {
        &mut self.fp
    }
}

#[derive(Debug, Clone)]
pub struct ItemSetRecord<M: Monoid> {
    range: Range<M::Item>,
    items: Vec<M::Item>,
    want_response: bool,
}

impl<M: Monoid> ItemSetRecord<M> {
    pub fn new(range: Range<M::Item>, items: Vec<M::Item>, want_response: bool) -> Self {
        Self {
            range,
            items,
            want_response,
        }
    }

    pub fn range(&self) -> &Range<M::Item> {
        &self.range
    }

    pub fn items(&self) -> &Vec<M::Item> {
        &self.items
    }

    pub fn want_response(&self) -> bool {
        self.want_response
    }
}

#[derive(Debug, Clone)]
pub struct Message<M: ProtocolMonoid + Encodable> {
    fps: Vec<FingerprintRecord<M>>,
    item_sets: Vec<ItemSetRecord<M>>,
}

impl<M> Message<M>
where
    M: ProtocolMonoid,
{
    pub fn new(fps: Vec<FingerprintRecord<M>>, item_sets: Vec<ItemSetRecord<M>>) -> Self {
        Self { fps, item_sets }
    }

    pub fn is_end(&self) -> bool {
        self.fps.is_empty() && self.item_sets.is_empty()
    }

    pub fn fingerprints(&self) -> &Vec<FingerprintRecord<M>> {
        &self.fps
    }

    pub fn item_sets(&self) -> &Vec<ItemSetRecord<M>> {
        &self.item_sets
    }
}

pub fn first_message<M, N>(root: &N) -> Result<Message<M>, EncodeError<M::EncodeError>>
where
    M: ProtocolMonoid,
    N: Node<M>,
{
    let msg = match root.node_contents() {
        Some(non_nil_node) => {
            let (min, max) = non_nil_node.bounds();
            let range = Range(min.clone(), max.next());
            let rev_range = range.reverse();
            let full_monoid = root.monoid().to_encoded()?;

            Message::new(
                vec![FingerprintRecord::new(range, full_monoid)],
                vec![ItemSetRecord::new(rev_range, vec![], true)],
            )
        }
        None => {
            let range = Range(M::Item::zero(), M::Item::zero());

            Message::new(vec![], vec![ItemSetRecord::new(range, vec![], true)])
        }
    };

    Ok(msg)
}

pub fn respond_to_message<M, N>(
    root: &N,
    msg: &Message<M>,
    threshold: usize,
    split: fn(usize) -> Vec<usize>,
) -> Result<(Message<M>, Vec<M::Item>), RespondError<M>>
where
    M: ProtocolMonoid,
    N: Node<M>,
{
    let mut response = Message::new(vec![], vec![]);
    let mut new_items = vec![];

    let dummy_encoded_fp = M::neutral().to_encoded()?;
    let mut prep_raw = vec![];
    let mut prep_parts = vec![];

    for item_set in msg.item_sets() {
        let ItemSetRecord {
            range,
            items,
            want_response,
        } = item_set;

        new_items.extend_from_slice(&items);
        if *want_response {
            let mut acc = ItemsAccumulator::new();
            root.query(range, &mut acc);
            response
                .item_sets
                .push(ItemSetRecord::new(range.clone(), acc.into_results(), false));
        }
    }

    for FingerprintRecord { range, fp } in msg.fingerprints() {
        let their_fp = M::from_encoded(fp)?;
        let mut my_fp_acc = SimpleAccumulator::new();
        root.query(range, &mut my_fp_acc);
        let my_fp = my_fp_acc.into_result();

        if my_fp != their_fp {
            if my_fp.count() < threshold {
                let mut acc = ItemsAccumulator::new();
                root.query(range, &mut acc);
                response.item_sets.push(ItemSetRecord::new(
                    range.clone(),
                    acc.into_results(),
                    true,
                ));
                continue;
            }

            let splits = split(my_fp.count());
            let mut acc = SplitAccumulator::new(range, &splits);
            root.query(range, &mut acc);
            let results = acc.results();
            let ranges = acc.ranges();
            for (i, fp) in results.iter().enumerate() {
                let sub_range = &ranges[i];
                if fp.count() < threshold {
                    let mut acc = ItemsAccumulator::new();
                    root.query(sub_range, &mut acc);
                    response.item_sets.push(ItemSetRecord::new(
                        sub_range.clone(),
                        acc.into_results(),
                        true,
                    ));
                } else {
                    prep_parts.push(FingerprintRecord::new(
                        sub_range.clone(),
                        dummy_encoded_fp.clone(),
                    ));
                    prep_raw.push(fp.clone());
                }
            }
        }
    }

    <M as Encodable>::batch_encode(&prep_raw, &mut prep_parts)?;
    response.fps.extend(prep_parts.into_iter());

    Ok((response, new_items))
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{collections::BTreeSet, format, vec, vec::Vec};

    extern crate std;
    use std::println;

    use crate::{
        easy::uniform::split as uniform_split, monoid::hashxor::CountingSha256Xor,
        tree::mem_rc::Node,
    };

    use proptest::{prelude::prop, prop_assert, proptest};

    proptest! {
        #[test]
        fn protocol_correctness(items_party_a in prop::collection::vec(1..1000u64, 1..100usize), items_party_b in prop::collection::vec(1..1000u64, 1..100usize)) {
            println!();

            let item_set_a: BTreeSet<u64> = BTreeSet::from_iter(items_party_a.iter().cloned());
            let item_set_b: BTreeSet<u64> = BTreeSet::from_iter(items_party_b.iter().cloned());

            println!("a items: {item_set_a:?}");
            println!("b items: {item_set_b:?}");

            let mut root_a: Node<CountingSha256Xor<u64>> = Node::nil();
            let mut root_b: Node<CountingSha256Xor<u64>> = Node::nil();

            for item in item_set_a.iter().cloned() {
                root_a = root_a.insert(item);
            }

            for item in item_set_b.iter().cloned() {
                root_b = root_b.insert(item);
            }

            let mut msg = super::first_message(&root_a).unwrap();

            let mut missing_items_a = vec![];
            let mut missing_items_b = vec![];

            loop {
                println!("a msg: {msg:?}");
                if msg.is_end() {
                    break
                }


            println!("b-----");
                let (resp, new_items) = super::respond_to_message(&root_b, &msg, 3, uniform_split::<2>).unwrap();
                missing_items_b.extend(new_items.into_iter());

                println!("b msg: {resp:?}");
                if resp.is_end() {
                    break
                }


            println!("a-----");
                let (resp, new_items) = super::respond_to_message(&root_a, &resp, 3, uniform_split::<2>).unwrap();
                missing_items_a.extend(new_items.into_iter());

                msg = resp;
            }

            println!("a all: {item_set_a:?} + {missing_items_a:?}");
            println!("b all: {item_set_b:?} + {missing_items_b:?}");

            let mut all_items = item_set_a.clone();
            let mut all_items_a = item_set_a.clone();
            let mut all_items_b = item_set_b.clone();
            all_items.extend(item_set_b.iter());
            all_items_a.extend(missing_items_a.iter());
            all_items_b.extend(missing_items_b.iter());

            let mut a_all: Vec<u64> = Vec::from_iter(all_items_a.iter().cloned());
            let mut b_all: Vec<u64> = Vec::from_iter(all_items_b.iter().cloned());
            let mut all: Vec<u64> = Vec::from_iter(all_items.iter().cloned());

            a_all.sort();
            b_all.sort();
            all.sort();

            println!("\n  all vec: {all:?}");
            println!("\na all vec: {a_all:?}, {:} {:}", a_all == all, all == a_all);
            println!("\nb all vec: {b_all:?}, {:} {:}", b_all == all, all == b_all);
            println!();

            let a_eq = a_all == all;
            let b_eq = b_all == all;

            println!("{a_eq}, {b_eq}");
            prop_assert!(a_eq, "a does not match");
            prop_assert!(b_eq, "a does not match");
        }
    }
}
