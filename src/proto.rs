use crate::{
    monoid::{Item, Monoid},
    query::{items::ItemsAccumulator, split::SplitAccumulator},
    range::Range,
    ranged_node::RangedNode,
};

pub trait ProtocolMonoid: Monoid {
    fn count(&self) -> usize;
}

#[derive(Debug, Clone)]
pub enum MessagePart<M: ProtocolMonoid> {
    Fingerprint(M),
    ItemSet(Vec<M::Item>, bool),
}

#[derive(Debug, Clone)]
pub struct Message<M: ProtocolMonoid>(Vec<(Range<M::Item>, MessagePart<M>)>)
where
    M::Item: Item;

impl<M> Message<M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    pub fn is_end(&self) -> bool {
        self.0.is_empty()
    }
}

pub fn first_message<M>(root: &RangedNode<M>) -> Message<M>
where
    M: ProtocolMonoid,
    M::Item: Item,
{
    Message(vec![
        (
            root.range().clone(),
            MessagePart::Fingerprint(root.node().monoid().clone()),
        ),
        (root.range().reverse(), MessagePart::ItemSet(vec![], true)),
    ])
}

pub fn respond_to_message<M: ProtocolMonoid>(
    root: &RangedNode<M>,
    msg: &Message<M>,
    threshold: usize,
    split: fn(usize) -> Vec<usize>,
) -> (Message<M>, Vec<M::Item>)
where
    M::Item: Item,
{
    let mut response_parts: Vec<(Range<_>, MessagePart<M>)> = vec![];
    let mut new_items = vec![];

    for (range, part) in &msg.0 {
        match part {
            MessagePart::Fingerprint(their_fp) => {
                let my_fp = root.query_range(range);
                if &my_fp != their_fp {
                    if my_fp.count() < threshold {
                        let mut acc = ItemsAccumulator::new();
                        root.query_range_generic(range, &mut acc);
                        response_parts.push((
                            range.clone(),
                            MessagePart::ItemSet(acc.into_results(), true),
                        ));
                        continue;
                    }

                    let splits = split(my_fp.count());
                    let mut acc = SplitAccumulator::new(range, &splits);
                    root.query_range_generic(range, &mut acc);
                    let results = acc.results();
                    let ranges = acc.ranges();
                    for (i, fp) in results.iter().enumerate() {
                        let sub_range = &ranges[i];
                        if fp.count() < threshold {
                            let mut acc = ItemsAccumulator::new();
                            root.query_range_generic(sub_range, &mut acc);
                            response_parts.push((
                                sub_range.clone(),
                                MessagePart::ItemSet(acc.into_results(), true),
                            ));
                        } else {
                            response_parts
                                .push((sub_range.clone(), MessagePart::Fingerprint(fp.clone())));
                        }
                    }
                }
            }
            MessagePart::ItemSet(items, want_response) => {
                new_items.extend_from_slice(&items);
                if *want_response {
                    let mut acc = ItemsAccumulator::new();
                    root.query_range_generic(range, &mut acc);
                    response_parts.push((
                        range.clone(),
                        MessagePart::ItemSet(acc.into_results(), false),
                    ));
                }
            }
        }
    }

    (Message(response_parts), new_items)
}

#[cfg(test)]
mod tests {
    use crate::{monoid::hashxor::CountingSha256Xor, range::Range, ranged_node::RangedNode, Node};
    use proptest::{prelude::prop, prop_assert, proptest};
    use std::collections::HashSet;

    proptest! {
        #[test]
        fn protocol_correctness(items_party_a in prop::collection::vec(1..1000u64, 1..100usize), items_party_b in prop::collection::vec(1..1000u64, 1..100usize)) {
            println!();

            let split = |n| {
                let snd = n / 2;
                let fst = n - snd;

                vec![fst, snd]
            };

            let item_set_a: HashSet<u64> = HashSet::from_iter(items_party_a.iter().cloned());
            let item_set_b: HashSet<u64> = HashSet::from_iter(items_party_b.iter().cloned());

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

            let min_a = item_set_a.iter().fold(1000, |acc, x| if *x < acc {*x} else {acc});
            let max_a = item_set_a.iter().fold(0, |acc, x| if *x > acc {*x} else {acc});
            let min_b = item_set_b.iter().fold(1000, |acc, x| if *x < acc {*x} else {acc});
            let max_b = item_set_b.iter().fold(0, |acc, x| if *x > acc {*x} else {acc});

            let ranged_root_a = RangedNode::new(&root_a, Range(min_a, max_a+1));
            let ranged_root_b = RangedNode::new(&root_b, Range(min_b, max_b+1));

            let mut msg = super::first_message(&ranged_root_a);

            let mut missing_items_a = vec![];
            let mut missing_items_b = vec![];

            loop {
                println!("a msg: {msg:?}");
                if msg.is_end() {
                    break
                }


                let (resp, new_items) = super::respond_to_message(&ranged_root_b, &msg, 3, split);
                missing_items_b.extend(new_items.into_iter());

                println!("b msg: {resp:?}");
                if resp.is_end() {
                    break
                }


                let (resp, new_items) = super::respond_to_message(&ranged_root_a, &resp, 3, split);
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

        let split = |n| {
            let snd = n / 2;
            let fst = n - snd;

            vec![fst, snd]
        };

        let item_set_a: HashSet<u64> = HashSet::from_iter(items_party_a.iter().cloned());
        let item_set_b: HashSet<u64> = HashSet::from_iter(items_party_b.iter().cloned());

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

        let min_a = item_set_a
            .iter()
            .fold(1000, |acc, x| if *x < acc { *x } else { acc });
        let max_a = item_set_a
            .iter()
            .fold(0, |acc, x| if *x > acc { *x } else { acc });
        let min_b = item_set_b
            .iter()
            .fold(1000, |acc, x| if *x < acc { *x } else { acc });
        let max_b = item_set_b
            .iter()
            .fold(0, |acc, x| if *x > acc { *x } else { acc });

        let ranged_root_a = RangedNode::new(&root_a, Range(min_a, max_a + 1));
        let ranged_root_b = RangedNode::new(&root_b, Range(min_b, max_b + 1));

        let mut msg = super::first_message(&ranged_root_a);

        let mut missing_items_a = vec![];
        let mut missing_items_b = vec![];

        loop {
            println!("a msg: {msg:?}");
            if msg.is_end() {
                break;
            }

            let (resp, new_items) = super::respond_to_message(&ranged_root_b, &msg, 3, split);
            missing_items_b.extend(new_items.into_iter());

            println!("b msg: {resp:?}");
            if resp.is_end() {
                break;
            }

            let (resp, new_items) = super::respond_to_message(&ranged_root_a, &resp, 3, split);
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
        assert!(b_eq, "a does not match");
    }
}
