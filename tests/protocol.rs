extern crate alloc;
use alloc::{collections::BTreeSet, vec, vec::Vec};

extern crate std;
use std::{io::Write, print, println};

use unionize::{
    easy::uniform::{split as uniform_split, Item as UniformItem, Node as UniformNode},
    protocol::{first_message, respond_to_message},
};

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

#[test]
fn sync_10k_msgs() {
    let mut shared_msgs = vec![UniformItem::default(); 6_000];
    let mut alices_msgs = vec![UniformItem::default(); 2_000];
    let mut bobs_msgs = vec![UniformItem::default(); 2_000];

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

    let mut msg = first_message(&alice_tree).unwrap();

    let mut missing_items_alice = vec![];
    let mut missing_items_bob = vec![];

    let mut count = 0;

    let loop_start_time = std::time::Instant::now();
    loop {
        count += 1;
        // println!("alice msg: {msg:?}");
        println!(
            "alice msg lengths: fps:{} item_sets:{}",
            msg.fingerprints().len(),
            msg.item_sets().len()
        );
        if msg.is_end() {
            break;
        }

        let (resp, new_items) = respond_to_message(&bob_tree, &msg, 3, uniform_split::<2>).unwrap();
        missing_items_bob.extend(new_items.into_iter());

        // println!("bob msg:   {resp:?}");
        println!(
            "bob msg lengths: fps:{} item_sets:{}",
            resp.fingerprints().len(),
            resp.item_sets().len()
        );
        if resp.is_end() {
            break;
        }

        let (resp, new_items) =
            respond_to_message(&alice_tree, &resp, 3, uniform_split::<2>).unwrap();
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

    let mut all_items_bob: BTreeSet<UniformItem> = BTreeSet::from_iter(shared_msgs.iter().cloned());
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
