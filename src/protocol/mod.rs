extern crate alloc;
use alloc::{vec, vec::Vec};

extern crate std;

pub mod encoding;
pub use encoding::{DecodeError, Encodable, EncodeError};

pub mod error;
pub use error::RespondError;

use serde::{Deserialize, Serialize};

use crate::{
    item::Item,
    monoid::Monoid,
    query::{
        item_filter::ItemFilterAccumulator, items::ItemsAccumulator, simple::SimpleAccumulator,
        split::SplitAccumulator,
    },
    range::Range,
    Node, NonNilNodeRef, Object, ObjectStore,
};

pub trait SerializableItem: Item + Serialize {}

pub trait ProtocolMonoid: Monoid + Encodable {
    fn count(&self) -> usize;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "M::Item: Serialize, for<'de2> M::Item: Deserialize<'de2>")]
pub struct Fingerprint<M>
where
    M: ProtocolMonoid,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
{
    range: Range<M::Item>,
    fp: M::Encoded,
}

impl<M> Fingerprint<M>
where
    M: ProtocolMonoid,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
{
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

impl<M> encoding::AsDestMutRef<M::Encoded> for Fingerprint<M>
where
    M: ProtocolMonoid,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
{
    fn as_dest_mut_ref(&mut self) -> &mut M::Encoded {
        &mut self.fp
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(bound = "M::Item: Serialize, for<'de2> M::Item: Deserialize<'de2>")]
pub struct ItemSet<M>
where
    M: ProtocolMonoid,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
{
    range: Range<M::Item>,
    items: Vec<M::Item>,
    want_response: bool,
}

impl<M: Monoid> ItemSet<M>
where
    M: ProtocolMonoid,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
{
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(bound = "M::Item: Serialize, for<'de2> M::Item: Deserialize<'de2>")]
pub struct Message<M, O>
where
    M: ProtocolMonoid,
    O: Object<M::Item> + Serialize,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
    for<'de2> O: Deserialize<'de2>,
{
    fps: Vec<Fingerprint<M>>,
    item_sets: Vec<ItemSet<M>>,
    wants: Vec<M::Item>,
    provide: Vec<O>,
}

impl<M, O> Message<M, O>
where
    M: ProtocolMonoid,
    O: Object<M::Item> + Serialize,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
    for<'de2> O: Deserialize<'de2>,
{
    pub fn new(
        fps: Vec<Fingerprint<M>>,
        item_sets: Vec<ItemSet<M>>,
        wants: Vec<M::Item>,
        provide: Vec<O>,
    ) -> Self {
        Self {
            fps,
            item_sets,
            wants,
            provide,
        }
    }

    pub fn is_end(&self) -> bool {
        self.fps.is_empty()
            && self.item_sets.is_empty()
            && self.wants.is_empty()
            && self.provide.is_empty()
    }

    pub fn fingerprints(&self) -> &Vec<Fingerprint<M>> {
        &self.fps
    }

    pub fn item_sets(&self) -> &Vec<ItemSet<M>> {
        &self.item_sets
    }

    pub fn wants(&self) -> &Vec<M::Item> {
        &self.wants
    }

    pub fn provide(&self) -> &Vec<O> {
        &self.provide
    }
}

pub fn first_message<O, M, N>(root: &N) -> Result<Message<M, O>, EncodeError<M::EncodeError>>
where
    M: ProtocolMonoid,
    O: Object<M::Item> + Serialize,
    N: Node<M>,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
    for<'de2> O: Deserialize<'de2>,
{
    let msg = match root.node_contents() {
        Some(non_nil_node) => {
            let (min, max) = non_nil_node.bounds();
            let range = Range(min.clone(), max.next());
            let rev_range = range.reverse();
            let full_monoid = root.monoid().to_encoded()?;

            Message::new(
                vec![Fingerprint::new(range, full_monoid)],
                vec![ItemSet::new(rev_range, vec![], true)],
                vec![],
                vec![],
            )
        }
        None => {
            let range = Range(M::Item::zero(), M::Item::zero());

            Message::new(
                vec![],
                vec![ItemSet::new(range, vec![], true)],
                vec![],
                vec![],
            )
        }
    };

    Ok(msg)
}

pub fn respond_to_message<O, M, N, S>(
    root: &N,
    object_store: &S,
    msg: &Message<M, O>,
    threshold: usize,
    split: fn(usize) -> Vec<usize>,
) -> Result<(Message<M, O>, Vec<O>), RespondError<M>>
where
    M: ProtocolMonoid,
    O: Object<M::Item> + Serialize,
    S: ObjectStore<M::Item, O>,
    N: Node<M>,
    M::Item: Serialize,
    M::Encoded: Serialize,
    for<'de2> M::Item: Deserialize<'de2>,
    for<'de2> M::Encoded: Deserialize<'de2>,
    for<'de2> O: Deserialize<'de2>,
{
    let mut fingerprints = vec![];
    let mut item_sets = vec![];
    let mut wants = vec![];

    let dummy_encoded_fp = M::neutral().to_encoded()?;
    let mut prep_raw = vec![];
    let mut prep_parts = vec![];

    let provide = object_store
        .get_batch(&msg.wants)
        .into_iter()
        .map(|opt_obj| opt_obj.unwrap().clone())
        .collect();

    for item_set in msg.item_sets() {
        let ItemSet {
            range,
            items,
            want_response,
        } = item_set;

        let mut dedup_acc = ItemFilterAccumulator::new(items);

        // query_range() returns None if there are no items, in which case we don't need to add
        // anything anyways
        if let Some(dedup_query_range) = dedup_acc.query_range() {
            root.query(&dedup_query_range, &mut dedup_acc);
            wants.extend(dedup_acc.result().cloned());
        }

        if *want_response {
            let mut acc = ItemsAccumulator::new();
            root.query(range, &mut acc);
            item_sets.push(ItemSet::new(range.clone(), acc.into_results(), false));
        }
    }

    for Fingerprint { range, fp } in msg.fingerprints() {
        let their_fp = M::from_encoded(fp)?;
        let mut my_fp_acc = SimpleAccumulator::new();
        root.query(range, &mut my_fp_acc);
        let my_fp = my_fp_acc.into_result();

        if my_fp != their_fp {
            if my_fp.count() < threshold {
                let mut acc = ItemsAccumulator::new();
                root.query(range, &mut acc);
                item_sets.push(ItemSet::new(range.clone(), acc.into_results(), true));
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
                    item_sets.push(ItemSet::new(sub_range.clone(), acc.into_results(), true));
                } else {
                    prep_parts.push(Fingerprint::new(
                        sub_range.clone(),
                        dummy_encoded_fp.clone(),
                    ));
                    prep_raw.push(fp.clone());
                }
            }
        }
    }

    <M as Encodable>::batch_encode(&prep_raw, &mut prep_parts)?;
    fingerprints.extend(prep_parts.into_iter());

    Ok((
        Message::new(fingerprints, item_sets, wants, provide),
        msg.provide.clone(),
    ))
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{
        collections::{BTreeMap, BTreeSet},
        format, vec,
        vec::Vec,
    };
    use xs233::{scalar::Scalar, xsk233::Xsk233Point, Point};

    extern crate std;
    use std::println;

    use crate::{
        easy::{
            tests::{TestMonoid, TestNode, TestObject},
            uniform::split as uniform_split,
        },
        item::le_byte_array::LEByteArray,
        monoid::{count::CountingMonoid, mulhash_xs233::MulHashMonoid},
        Monoid, Range,
    };

    use proptest::{prelude::prop, prop_assert, prop_assert_eq, prop_compose, proptest};

    use super::{Encodable, Fingerprint, ItemSet, Message};

    proptest! {
        #[test]
        fn protocol_correctness(items_party_a in prop::collection::vec(1..1000u64, 1..100usize), items_party_b in prop::collection::vec(1..1000u64, 1..100usize)) {
            println!("---test run---");

            let item_set_a: BTreeSet<u64> = BTreeSet::from_iter(items_party_a.iter().cloned());
            let item_set_b: BTreeSet<u64> = BTreeSet::from_iter(items_party_b.iter().cloned());

            let intersection: Vec<_> = item_set_a.intersection(&item_set_b).cloned().collect();

            println!("a items: {item_set_a:?}");
            println!("b items: {item_set_b:?}");

            let mut root_a = TestNode::nil();
            let mut root_b = TestNode::nil();

            let mut object_store_a = BTreeMap::new();
            let mut object_store_b = BTreeMap::new();

            for item in item_set_a.iter().cloned() {
                root_a = root_a.insert(item);
                object_store_a.insert(item, (item, true));
            }

            for item in item_set_b.iter().cloned() {
                root_b = root_b.insert(item);
                object_store_b.insert(item, (item, true));
            }

            let mut msg:Message<TestMonoid, TestObject> = super::first_message(&root_a).unwrap();

            let mut missing_items_a = vec![];
            let mut missing_items_b = vec![];

            loop {
                println!("a msg: {msg:?}");
                if msg.is_end() {
                    break
                }

                println!("b-----");
                let (resp, new_objects) = super::respond_to_message(&root_b, &object_store_b, &msg, 3, uniform_split::<2>).unwrap();
                missing_items_b.extend(new_objects.into_iter().map(|(item, _)| item));

                println!("b msg: {resp:?}");
                if resp.is_end() {
                    break
                }

                println!("a-----");
                let (resp, new_objects) = super::respond_to_message(&root_a, &object_store_a, &resp, 3, uniform_split::<2>).unwrap();
                missing_items_a.extend(new_objects.into_iter().map(|(item, _)| item));

                msg = resp;
            }

            println!("a all: {item_set_a:?} + {missing_items_a:?}");
            println!("b all: {item_set_b:?} + {missing_items_b:?}");

            prop_assert_eq!(missing_items_a.len(), item_set_b.len() - intersection.len());
            prop_assert_eq!(missing_items_b.len(), item_set_a.len() - intersection.len());


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

    prop_compose! {
        fn arb_item()
            (item_bs in proptest::array::uniform30(0u8..=255u8)) -> LEByteArray<30> {
                LEByteArray(item_bs)
            }
    }

    prop_compose! {
        fn arb_range()
            (from in arb_item(), to in arb_item()) -> Range<LEByteArray<30>> {
            Range(from, to)
        }
    }

    prop_compose! {
        fn arb_scalar()
            (scalar in proptest::array::uniform29(0u8..=255u8)) -> Scalar<29> {
                Scalar::new(scalar)
            }
    }

    prop_compose! {
        fn arb_point()
            (scalar in arb_scalar()) -> xs233::xsk233::Xsk233Point {
                let mut pt = <xs233::xsk233::Xsk233Point as xs233::Point>::generator().clone();
                pt.mul_inplace(&scalar);
                pt
            }
    }

    prop_compose! {
        fn arb_fp_rec()
            (range in arb_range(), fp in arb_point(), count in 0..1432usize) -> Fingerprint<CountingMonoid<MulHashMonoid<Xsk233Point>>> {
                let mut monoid = MulHashMonoid::neutral();
                monoid.set(fp);
                Fingerprint{
                    range,
                    fp: CountingMonoid::new(count, monoid).to_encoded().unwrap() ,
                }
            }
    }

    prop_compose! {
        fn arb_item_set_rec()
            (range in arb_range(), items in proptest::collection::vec(arb_item(), 0..10), want_response in proptest::bool::ANY) -> ItemSet<CountingMonoid<MulHashMonoid<Xsk233Point>>> {
                ItemSet {
                    range,
                    items,
                    want_response
                }
            }
    }

    prop_compose! {
        fn arb_message()
            (fps in proptest::collection::vec( arb_fp_rec(), 0..10), item_sets in proptest::collection::vec(arb_item_set_rec(), 0..10)) -> Message<CountingMonoid<MulHashMonoid<Xsk233Point>>, (LEByteArray<30>, bool)>{
                Message{
                    fps, item_sets, wants: vec![], provide: vec![]
                }
            }
    }

    proptest! {
        #[test]
        fn serialize_correctness_stream(msg in arb_message()) {
            let mut buffer = Vec::new();
            serde_cbor::to_writer(&mut buffer, &msg).unwrap();
            let result = serde_cbor::from_reader(&buffer[..]).unwrap();
            prop_assert_eq!(msg, result);

        }
    }

    proptest! {
        #[test]
        fn serialize_correctness(msg in arb_message()) {
            println!("m:{msg:?}");
            let encoded = serde_cbor::to_vec(&msg).unwrap();
            println!("e:{encoded:x?}");
            let result = serde_cbor::from_slice(&encoded).unwrap();
            prop_assert_eq!(msg, result);

        }
    }
}
