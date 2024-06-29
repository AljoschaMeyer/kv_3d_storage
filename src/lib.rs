//! An efficient three-dimensional data structure, backed by any key-value store.
//! 
//! ## Architecture
//! 
//! The `point3d` module defines `Point3d`, a point in a three-dimensional space. It defines three total orders, which correspond to lexicographically comparing the three dimensions in order of xyz, yzx, and zxy espectively.
//! 
//! We store points in a key-value store. We assume the key-value store to use lexicographically sorted bytestrings as its keys. The precise interface we require of the underlying storage backend is defined in the `backend` module.
//! 
//! For efficient operation, we encode points in a way such that lexicographically comparing the encodins of two points yields the same result as corresponding the points themselves. Such *order-homomorphic encodings* (just *homomorphic encodings* for short) form the basis of efficiently using a kv store. The `Dimension` trait in the `point3d` module defines the requirements that individual dimensions and their encodings must fulfil so that `Point3d` can provide a homomorphic encoding for xyz, yzx, and zxy orderings each.
//! 
//! We need those three different orderings because we implement a [kd-tree](https://en.wikipedia.org/wiki/K-d_tree)-ish data structure on top of the kv store. We combine the idea behind kv trees with that of [zip-trees](https://stackoverflow.com/a/61944199). You will want to read up on both zip-trees and kd-trees in order to follow the next definitions. Crucially, we do not cycle through dimension orderings based on node depth, but based on node *rank*.
//! 
//! More precisely: we define the 3d-ish-zip-tree on a set of pairs of `Point3d`s and corresponding *rank* (natural numbers, including zero) as the unique binary tree that upholds the following properties:
//! 
//! - the rank of any left child vertex is strictly less than the rank of its parent,
//! - the rank of any right child vertex is less than or equal to the rank of its parent,
//! - for any vertex `v` whose rank is divisible by three with remainder two:
//!     - all items in the subtree rooted at the left child of `v` are strictly less than the item of `v` in the xyz ordering, and
//!     - all items in the subtree rooted at the right child of `v` are strictly greater than the item of `v` in the xyz ordering, and 
//! - for any vertex `v` whose rank is divisible by three with remainder one:
//!     - all items in the subtree rooted at the left child of `v` are strictly less than the item of `v` in the yzx ordering, and
//!     - all items in the subtree rooted at the right child of `v` are strictly greater than the item of `v` in the yzx ordering, and 
//! - for any vertex `v` whose rank is divisible by three with remainder zero:
//!     - all items in the subtree rooted at the left child of `v` are strictly less than the item of `v` in the zxy ordering, and
//!     - all items in the subtree rooted at the right child of `v` are strictly greater than the item of `v` in the zxy ordering.
//! 
//! In typical zip-tree fashion, this definition is equivalent to sequentially inserting item-rank pairs into a tree without rebalancing, where the tree is a 3d-ish search tree (i.e., a 3d-tree, but with ordering based on rank, not depth), if the sequence is sorted in order of descending ranks (and, within groups of equal rank, sorted ascending according to the order that corresponds to the rank).
//! 
//! We do not make use of the unique shape of these trees to obtain the desired semantics of our 3d item store, but we *do* utilise the unique shape for testing: we define a trivially correct version of 3d-ish-zip-trees based on the characterisation of sequential insertions without rebalancing, and we use this in fuzz tests to ensure that operating on our proper, kv-based implementation always yields the same results as constructing the control tree.
//! 
//! To allow to efficiently answer certain queries, all our trees are [monoid trees](https://github.com/AljoschaMeyer/rbsr_short/blob/main/main.pdf), based off the [`LiftingCommutativeMonoid` trait](monoid::LiftingCommutativeMonoid). Monoids must be commutative, or things will randomly break. We always employ the counting monoid, plus an arbitrary user-specified monoid.
//! 
//! TODO mapping of tree to set of kv entries


mod point3d;
pub use point3d::*;

mod backend;
pub use backend::*;

mod monoid;
pub use monoid::*;




