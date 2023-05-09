# set *recon*ciliation

a first shot at the set reconciliation algorithm in rust. you know the idea, so i'll just give a brief walkthrough through the code.

the things a user of this crate roughly has to do:
- decide on (or build their own) monoid
- make their data implement the `Item` trait
- put their items into a tree
- run the protocol

so far, this crate does not ship with very good monoids, but at least with strong trust assumptions, they are easy to implement (just make the hashxor monoid better, use that for now). If you do not have these, you need more crypto, which is planned for ᴛʜᴇ ғᴜᴛᴜʀᴇ.

> the monoid traits are in `monoid/mod.rs`. the other files in that directory are specific monoids. one of them, `SumMonoid`, requires that your items have a zero value and can be added, as per the `SumItem` trait defined in the respective file.

your data (probably just hashes) needs to implement the `Item` trait. the trait is just `Clone + Ord + Debug`. go and implement those, and then add an empty `impl` block for `Item`.

> the `Item` trait also is defined in `monoid/mod.rs`.

create a new tree using `Node::nil()`. then insert your data using the `insert` method. a call to insert returns a new tree, so do something like `tree = tree.insert(stuff);`.

> the `Node` type is defined in `tree/mod.rs`. since this is a 2-3 tree, we can have different types of nodes, which is why we have the `NodeData` enum, defined in the same file. the code for insertion is in `tree/insert.rs`. this is where the growing and splitting of `NodeData` values happens, basically the 2-3 tree logic.

now you just have to run the protocol. the protocol is defined as a method of the `RangedNode` type. this is a thin wrapper around a `Node` that annotates it with a `Range`. this makes queries a lot easier. getting a `RangedNode` is actually a bit of a pain, and if you have better ideas for doing this i'm all ears, but here's what i got. a `Range` is basically the sort of range as defined in the paper: a tuple of items (from, to), and if from < to, then an item is considered part of the range if from <= item  && item < to. if from >= to, it is considered part of the range if from <= item || item < to. this definition handles wrapping and the concept that two identical items denote the full range, as in the paper. so to get the range for the tree at hand, you have to search for the min and max of your items and then us the range (min, max+1). because using max would mean that your largest item si not part of the range. that means that you basically have to implement adding or at least a "plus 1" operation for your items. sorry about that. anyway, so you create a new `RangedNode` using `RangedNode::new(&tree_root_node, range)`. do all of that for both parties. at one party call the `first_message` function and the give the other party the result of that and pass it into the respond_to_message function, which returns a new message and the items it learned about. pass the messages back and forth until one of them returns an empty message. for such messages the `is_end` method will return true.

> wow that was a very long paragraph, i hope it made sense. `Range` is defined in `range.rs`, `RangedNode` is defined in `ranged_node.rs`, and all the protocol functions are in `proto.rs`. these should probably be exported in `lib.rs`? not sure...anyway. one thing i didn't touch on here is the querying behind the scenes. this happens in the query directory and there is a little bit i can talk about. the main function is `query_range_generic`. there used to be others that were less generic, but they are gone, so maybe that function should be renamed. anyway, the trick is that it takes a mutable borrow to a value that implement the `Accumulator` trait, defined in `query/mod.rs`. this allows us to just get a fingerprint for the range (`query/simple.rs`), to get several fingerprints, split by subrange item count (`query/split.rs`) or to get all the items in the range (`query/items.rs`).

for a little bit of guidance, take a look at the `protocol_correctness` test in `proto.rs`. it's not much, but it is what it is.

i hope this helps!
