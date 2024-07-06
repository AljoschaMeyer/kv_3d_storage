//! This module defines how to represent monoid-3d-ish-zip-trees on a kv store, in the form of a *kv-tree*.
//! 
//! For any zip-tree vertex for point `p`, rank `r`, value `v`, and monoidal summary data `s`, we add to the kv-store an entry consisting of
//! 
//! - the key, the concatenation of the `r` (encoded as a single byte) and the appropriate homomorphic encoding of `p`:
//!     - xyz-encoding if `r % 3 == 2`
//!     - yzx-encoding if `r % 3 == 1`
//!     - zxy-encoding if `r % 3 == 0`
//! - the value, which consists of
//!     - `r`, `v`, and `s`,
//!     - the rank of the left child of the vertex, or 255 if there is no left child
//!     - the rank of the right child of the vertex, or 255 if there is no right child
//! 
//! With this information, we can efficiently find the left or right child of any given vertex.
//! 
//! To find the left child: given a zip-tree vertex for point `p` and left-child-rank `lr`, let `enc` be the homomorphic encoding of `p` for the rank `lr` (**not its own rank**). Querying the kv-store for the greatest key that is strictly less than the concatenation of `lr` and `enc` then yields the left child.
//! 
//! To find the right child: given a zip-tree vertex for point `p` and left-child-rank `rr`, let `enc` be the homomorphic encoding of `p` for the rank `rr` (**not its own rank**). Querying the kv-store for the least key that is strictly greater than the concatenation of `rr` and `enc` then yields the right child.