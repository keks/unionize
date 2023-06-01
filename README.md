# Unionize!

Range-based Set Reconciliation in Rust. A protocol that allows two parties that have one set each to efficiently get the union of the two sets.

The core idea is that we have fingerprints that can be combined. That means that if I have one fingerprint for the set `{A, B}` and one for `{C, D}`. I can compute the figerprint for {A, B, C, D} without having to add each item individually. Because of this property, we call the fingerprints `Monoid` in this crate. There are different ways to construct a monoid, which all have different consequences. The one in `mulhash_xs233` should be secure against censorship attacks, which the others aren't!

It also is important to consider what `A`, `B`, `C` and `D` are. We call them `Item`s and only place a few constraints on them: `Copy`, `Debug` and `Ord`. In order to use use the protocol you'll also need to implement `Peano`, which means you need a `zero` function and a function that returns the element after the current one (in the order used by `Ord`). Often, they will be the hash of the actual set member.

In order to use the protocol, the items need to be in a tree. The tree lives inside a `Node`, which you can add items into. Note that in order to keep the datastructure pure, you have to keep the return value of the `insert` function.

You need to wrap the `Node` with a `RangedNode`. Take a look at the tests how that is done. This is needed because the protocol needs to always know the smallest and largest element of any subtree.

Finally, use the `first_message` and `respond_to_message` functions in the `proto` module to run the protocol. Getting the message to the other party is your business (:
