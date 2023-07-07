extern crate alloc;
use alloc::{vec, vec::Vec};

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

pub trait Encodable: Default {
    type Encoded: Clone + core::fmt::Debug + Eq + Default;
    type Error;

    fn encode(&self, encoded: &mut Self::Encoded) -> Result<(), Self::Error>;
    fn decode(&mut self, encoded: &Self::Encoded) -> Result<(), Self::Error>;

    fn to_encoded(&self) -> Result<Self::Encoded, Self::Error> {
        let mut encoded = Self::Encoded::default();
        self.encode(&mut encoded)?;
        Ok(encoded)
    }

    fn from_encoded(encoded: &Self::Encoded) -> Result<Self, Self::Error> {
        let mut decoded = Self::default();
        decoded.decode(encoded)?;
        Ok(decoded)
    }
}

#[derive(Debug, Clone)]
pub enum MessagePart<M: ProtocolMonoid> {
    Fingerprint(M::Encoded),
    ItemSet(Vec<M::Item>, bool),
}

#[derive(Debug, Clone)]
pub struct Message<M: ProtocolMonoid + Encodable>(Vec<(Range<M::Item>, MessagePart<M>)>);

impl<M> Message<M>
where
    M: ProtocolMonoid,
{
    pub fn is_end(&self) -> bool {
        self.0.is_empty()
    }
}

pub fn first_message<M, N>(root: &N) -> Result<Message<M>, M::Error>
where
    M: ProtocolMonoid,
    N: Node<M>,
{
    let parts = match root.node_contents() {
        Some(non_nil_node) => {
            let (min, max) = non_nil_node.bounds();
            let range = Range(min.clone(), max.next());
            let full_monoid = MessagePart::Fingerprint(root.monoid().to_encoded()?);
            vec![
                (range.clone(), full_monoid),
                (range.reverse(), MessagePart::ItemSet(vec![], true)),
            ]
        }
        None => {
            vec![(
                Range(M::Item::zero(), M::Item::zero()),
                MessagePart::ItemSet(vec![], true),
            )]
        }
    };

    Ok(Message(parts))
}

pub fn respond_to_message<M, N>(
    root: &N,
    msg: &Message<M>,
    threshold: usize,
    split: fn(usize) -> Vec<usize>,
) -> Result<(Message<M>, Vec<M::Item>), M::Error>
where
    M: ProtocolMonoid,
    N: Node<M>,
{
    let mut response_parts: Vec<(Range<_>, MessagePart<M>)> = vec![];
    let mut new_items = vec![];

    for (range, part) in &msg.0 {
        match part {
            MessagePart::Fingerprint(their_fp) => {
                let their_fp = M::from_encoded(their_fp)?;
                let mut my_fp_acc = SimpleAccumulator::new();
                root.query(range, &mut my_fp_acc);
                let my_fp = my_fp_acc.into_result();
                if my_fp != their_fp {
                    if my_fp.count() < threshold {
                        let mut acc = ItemsAccumulator::new();
                        root.query(range, &mut acc);
                        response_parts.push((
                            range.clone(),
                            MessagePart::ItemSet(acc.into_results(), true),
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
                            response_parts.push((
                                sub_range.clone(),
                                MessagePart::ItemSet(acc.into_results(), true),
                            ));
                        } else {
                            response_parts.push((
                                sub_range.clone(),
                                MessagePart::Fingerprint(fp.to_encoded()?),
                            ));

                            // if fp.count() == threshold {
                            //     print!("items ");
                            //     let mut acc = ItemsAccumulator::new();
                            //     root.query(sub_range, &mut acc);
                            // }
                        }
                    }
                    // } else {
                    // let mut acc = ItemsAccumulator::new();
                    // root.query(range, &mut acc);
                }
            }
            MessagePart::ItemSet(items, want_response) => {
                new_items.extend_from_slice(&items);
                if *want_response {
                    let mut acc = ItemsAccumulator::new();
                    root.query(range, &mut acc);
                    response_parts.push((
                        range.clone(),
                        MessagePart::ItemSet(acc.into_results(), false),
                    ));
                }
            }
        }
    }

    Ok((Message(response_parts), new_items))
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{collections::BTreeSet, format, vec, vec::Vec};

    extern crate std;
    use std::{io::Write, print, println};

    use crate::{
        easy::uniform::{split as uniform_split, Item as UniformItem, Node as UniformNode},
        monoid::hashxor::CountingSha256Xor,
        tree::mem_rc::Node,
    };

    use proptest::{prelude::prop, prop_assert, proptest};
    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn sync_100k_msgs() {
        let mut shared_msgs = vec![UniformItem::default(); 60_000];
        let mut alices_msgs = vec![UniformItem::default(); 20_000];
        let mut bobs_msgs = vec![UniformItem::default(); 20_000];

        let mut alice_tree = UniformNode::nil();
        let mut bob_tree = UniformNode::nil();

        let statm = procinfo::pid::statm_self().unwrap();
        println!("current memory usage: {statm:#?}");

        let gen_start_time = std::time::Instant::now();

        print!("generating and adding items... ");
        std::io::stdout().flush().unwrap();
        let mut rng = ChaCha8Rng::from_seed([23u8; 32]);
        for msg in &mut shared_msgs {
            rng.fill(&mut msg.0);
            alice_tree = alice_tree.insert(msg.clone());
            bob_tree = bob_tree.insert(msg.clone());
        }
        for msg in &mut alices_msgs {
            rng.fill(&mut msg.0);
            alice_tree = alice_tree.insert(msg.clone());
        }
        for msg in &mut bobs_msgs {
            rng.fill(&mut msg.0);
            bob_tree = bob_tree.insert(msg.clone());
        }
        println!("done after {:?}.", gen_start_time.elapsed());
        // println!("shared messages: {shared_msgs:?}\n");
        // println!("alices messages: {alices_msgs:?}\n");
        // println!("bobs messages: {bobs_msgs:?}\n");
        // println!("alices tree: {alice_tree:?}");
        std::io::stdout().flush().unwrap();

        let statm = procinfo::pid::statm_self().unwrap();
        println!("current memory usage: {statm:#?}");

        let mut msg = super::first_message(&alice_tree).unwrap();

        let mut missing_items_alice = vec![];
        let mut missing_items_bob = vec![];

        let mut count = 0;

        let loop_start_time = std::time::Instant::now();
        loop {
            count += 1;
            // println!("alice msg: {msg:?}");
            println!("alice msg length: {}", msg.0.len());
            if msg.is_end() {
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&bob_tree, &msg, 3, uniform_split::<2>).unwrap();
            missing_items_bob.extend(new_items.into_iter());

            // println!("bob msg:   {resp:?}");
            println!("bob msg length:   {}", resp.0.len());
            if resp.is_end() {
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&alice_tree, &resp, 3, uniform_split::<2>).unwrap();
            missing_items_alice.extend(new_items.into_iter());

            msg = resp;
        }

        println!(
            "protocol took {count} rounds and {:?}.",
            loop_start_time.elapsed()
        );

        let mut all_items: BTreeSet<UniformItem> = BTreeSet::from_iter(shared_msgs.iter().cloned());
        all_items.extend(alices_msgs.iter());
        all_items.extend(bobs_msgs.iter());

        let mut all_items_alice: BTreeSet<UniformItem> =
            BTreeSet::from_iter(shared_msgs.iter().cloned());
        all_items_alice.extend(alices_msgs.iter());

        let mut all_items_bob: BTreeSet<UniformItem> =
            BTreeSet::from_iter(shared_msgs.iter().cloned());
        all_items_bob.extend(bobs_msgs.iter());

        all_items_alice.extend(missing_items_alice.iter());
        all_items_bob.extend(missing_items_bob.iter());

        // let bob_lacks: Vec<_> = all_items.difference(&all_items_bob).collect();
        // println!("bob lacks {} messages: {bob_lacks:?}", bob_lacks.len());
        // let bob_superfluous: Vec<_> = all_items_bob.difference(&all_items).collect();
        // println!("bob has too many: {bob_superfluous:?}");

        let all_len = all_items.len();
        let alice_all_len = all_items_alice.len();
        let bob_all_len = all_items_bob.len();

        println!("lens: all:{all_len} alice:{alice_all_len}, bob:{bob_all_len}");
        assert_eq!(all_len, alice_all_len);
        assert_eq!(all_len, bob_all_len);

        let mut all: Vec<_> = Vec::from_iter(all_items.iter().cloned());
        let mut alice_all: Vec<_> = Vec::from_iter(all_items_alice.iter().cloned());
        let mut bob_all: Vec<_> = Vec::from_iter(all_items_bob.iter().cloned());

        alice_all.sort();
        bob_all.sort();
        all.sort();

        // println!("\n  all vec: {all:?}");
        // println!(
        //     "\na all vec: {alice_all:?}, {:} {:}",
        //     alice_all == all,
        //     all == alice_all
        // );
        // println!(
        //     "\nb all vec: {bob_all:?}, {:} {:}",
        //     bob_all == all,
        //     all == bob_all
        // );
        // println!();

        let alice_eq = alice_all == all;
        let bob_eq = bob_all == all;

        println!("{alice_eq}, {bob_eq}");
        assert!(alice_eq, "a does not match");
        assert!(bob_eq, "a does not match");
    }

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

    #[test]
    fn repro_5() {
        let items_party_a = vec![
            569, 471, 225, 536, 674, 343, 719, 553, 973, 664, 866, 492, 693, 508, 256, 795, 447,
            939, 318, 880, 662, 571, 626, 816, 711, 421, 374, 955, 920, 972, 11, 257, 772,
            996, // 996
            917, 690, 989, 851, 454, 533, 709, 496, 366, 550, 980, 986, 889, // 989, 986
            525, 716, 766, 221, 275, 933, 278, 936, 917, 404, 766, 684, 25, 287, 7, 240, 136, 227,
            264, 230, 210, 962, 291, 870, 683, 585, 534, 14, 841, 755, 334, 92, 10, 778, 47, 570,
            61, 559, 35, 908, 258, 276, 562, 581, 175, 768, 263, 440, 117, 266, 541,
        ];
        let items_party_b = vec![
            826, 331, 743, 143, 474, 433, 64, 762, 916, 992, 855, 17, 889, 263, 963, // 992
            860, 366, 137, 691, 522, 623, 62, 198, 765, 887, 660, 11, 603, 584, 54, 744, 181, 742,
            28, 830, 230, 995, 684, 433, 952, 429, 875, 464, 849, 271, 891, 714, 967, // 995
            828, 530, 464, 888, 830, 182, 269, 724, 369, 266, 431, 425, 389, 412, 784, 865, 984,
            839, 936, 878, 846, 173, // 984
            631, 847, 983, 944, 9, 79, 915, 548, 521, 254, 441, 526, 8,
        ];

        // the protocol sends a fingerprint for 984..997. the protocol thinks that both parties
        // have the same, since the fingerprints match (because the sums of the values in range are
        // the same, and they have the same count - 3).
        // This was fixed by using the HashXorSha256 monoid instead of TestMonoid (which was just
        // adding the numbers)

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
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&root_b, &msg, 3, uniform_split::<2>).unwrap();
            missing_items_b.extend(new_items.into_iter());

            println!("b msg: {resp:?}");
            if resp.is_end() {
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&root_a, &resp, 3, uniform_split::<2>).unwrap();
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
        println!(
            "\na all vec: {a_all:?}, {:} {:}",
            a_all == all,
            all == a_all
        );
        println!(
            "\nb all vec: {b_all:?}, {:} {:}",
            b_all == all,
            all == b_all
        );
        println!();

        let a_eq = a_all == all;
        let b_eq = b_all == all;

        println!("{a_eq}, {b_eq}");
        assert!(a_eq, "a does not match");
        assert!(b_eq, "b does not match");
    }

    #[test]
    fn repro_6() {
        let items_party_a = vec![1, 20];
        let items_party_b = vec![3, 21, 1, 2];

        // the protocol sends a fingerprint for 984..997. the protocol thinks that both parties
        // have the same, since the fingerprints match (because the sums of the values in range are
        // the same, and they have the same count - 3).
        // This was fixed by using the HashXorSha256 monoid instead of TestMonoid (which was just
        // adding the numbers)

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
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&root_b, &msg, 3, uniform_split::<2>).unwrap();
            missing_items_b.extend(new_items.into_iter());

            println!("b msg: {resp:?}");
            if resp.is_end() {
                break;
            }

            let (resp, new_items) =
                super::respond_to_message(&root_a, &resp, 3, uniform_split::<2>).unwrap();
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
        println!(
            "\na all vec: {a_all:?}, {:} {:}",
            a_all == all,
            all == a_all
        );
        println!(
            "\nb all vec: {b_all:?}, {:} {:}",
            b_all == all,
            all == b_all
        );
        println!();

        let a_eq = a_all == all;
        let b_eq = b_all == all;

        println!("{a_eq}, {b_eq}");
        assert!(a_eq, "a does not match");
        assert!(b_eq, "b does not match");
    }
}
