use core::cmp::Ordering;
use core::fmt::Debug;
use core::hash::Hash;
use std::collections::HashSet;

use arbitrary::Arbitrary;

use kv_3d_storage::*;

/// A `u8` that uses a fixed-width homomorphic encoding.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Arbitrary)]
pub struct U8FixedWidth(pub u8);

impl Dimension for U8FixedWidth {
    const HOMOMORPHIC_ENCODING_MAX_LENGTH: usize = 1;

    const IS_FIXED_WIDTH_ENCODING: bool = true;

    fn homomorphic_encode(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.0;
        return 1;
    }

    fn homomorphic_decode(buf: &[u8]) -> Result<(Self, usize), ()> {
        if buf.len() == 0 {
            return Err(());
        } else {
            return Ok((U8FixedWidth(buf[0]), 1));
        }
    }
}

/// A `u8` that uses a variable-width homomorphic encoding.
///
/// The encoding of a `u8` `n` consists of `n` times the byte `0x02`, followed by the single byte `0x01`.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash, Arbitrary)]
pub struct U8VariableWidth(pub u8);

impl Dimension for U8VariableWidth {
    const HOMOMORPHIC_ENCODING_MAX_LENGTH: usize = 256;

    const IS_FIXED_WIDTH_ENCODING: bool = false;

    fn homomorphic_encode(&self, buf: &mut [u8]) -> usize {
        let n = self.0 as usize;
        for i in 0..n {
            buf[i] = 2;
        }
        buf[n] = 1;

        return n + 1;
    }

    fn homomorphic_decode(buf: &[u8]) -> Result<(Self, usize), ()> {
        let mut i = 0;
        while buf[i] != 1 {
            if i >= 256 {
                return Err(());
            }

            if buf[i] == 2 {
                i += 1;
            } else {
                return Err(());
            }
        }

        return Ok((Self(i as u8), i + 1));
    }
}

/// An in-memory control implementation of a 3d-ish-zip-tree.
///
/// X, Y, Z are the three dimensions.
/// V is the type of values to which the Point3ds are mapped.
/// M is the monoid for summarizing information about the point-value pairs.
// This testing utility is important enough to warrant its own tests: fuzz/control.rs
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum ControlNode<X, Y, Z, V, M>
where
    X: Dimension + Clone + Debug,
    Y: Dimension + Clone + Debug,
    Z: Dimension + Clone + Debug,
    V: Debug + Clone,
    M: LiftingCommutativeMonoid<(Point3d<X, Y, Z>, V)> + Debug,
{
    Empty,
    NonEmpty {
        key: Point3d<X, Y, Z>,
        rank: u8,
        left: Box<Self>,
        right: Box<Self>,
        value: V,
        // Total number of non-empty nodes in the tree rooted at this node.
        count: usize,
        // Accumulated monoidal value over the tree rooted at this node.
        summary: M,
    },
}

impl<X, Y, Z, V, M> ControlNode<X, Y, Z, V, M>
where
    X: Dimension + Clone + Debug + Hash,
    Y: Dimension + Clone + Debug + Hash,
    Z: Dimension + Clone + Debug + Hash,
    V: Debug + Clone,
    M: LiftingCommutativeMonoid<(Point3d<X, Y, Z>, V)> + Debug,
{
    /// Create a control tree from a set of points, associated values, and desired ranks.
    /// In case of duplicate points, will ignore all but one of them. Which one will be preserved
    /// is unspecified -> ensure to only use this without duplicate points.
    pub fn from_iter<I: Iterator<Item = (Point3d<X, Y, Z>, V, u8)>>(iter: I) -> Self {
        let mut sorted: Vec<_> = iter.collect();

        // Before we sort, remove all but the first occurence of each point.
        let mut uniques = HashSet::new();
        sorted.retain(|(point, _, _)| uniques.insert(point.clone()));

        // Sort by descending rank, and ascending according to the rank-appropriate order within each rank.
        sorted.sort_by(|(p1, _, rank1), (p2, _, rank2)| {
            match rank2.cmp(rank1) {
                Ordering::Equal => return cmp_points_at_rank(*rank1, p1, p2),
                _ => return rank2.cmp(rank1), // The unintuitive ordering results in *descending* sorting.
            }
        });

        let mut tree = ControlNode::Empty;
        for (point, value, rank) in sorted {
            tree.insert_no_balance(point, value, rank);
        }

        return tree;
    }

    // Insert point-value pair without rebalancing.
    fn insert_no_balance(&mut self, point: Point3d<X, Y, Z>, value: V, rank: u8) {
        let kv_pair = (point, value);
        let summary = M::lift(&kv_pair);
        let (point, value) = kv_pair;

        match self {
            ControlNode::Empty => {
                *self = ControlNode::NonEmpty {
                    key: point,
                    rank: rank,
                    left: Box::new(ControlNode::Empty),
                    right: Box::new(ControlNode::Empty),
                    value: value,
                    count: 1,
                    summary: summary,
                }
            }
            ControlNode::NonEmpty {
                key: parent_key,
                rank: parent_rank,
                left,
                right,
                ref mut count,
                summary: parent_summary,
                ..
            } => {
                match cmp_points_at_rank(*parent_rank, parent_key, &point) {
                    Ordering::Equal => {
                        panic!("Do not insert duplicate points into a control tree.")
                    }
                    Ordering::Less => {
                        right.insert_no_balance(point, value, rank);
                    }
                    Ordering::Greater => {
                        left.insert_no_balance(point, value, rank);
                    }
                }

                *count = *count + 1;
                *parent_summary = M::combine(parent_summary, &summary);
            }
        }
    }

    /// Panic if self is not a valid 3d-ish-zip-tree.
    /// This is for testing purposes, and *should* never panic...
    pub fn assert_tree_invariants(&self) {
        self.do_assert_tree_invariants();
    }

    fn do_assert_tree_invariants(
        &self,
    ) -> (
        Option<Point3d<X, Y, Z>>, /* min contained point in xyz ordering */
        Option<Point3d<X, Y, Z>>, /* max contained point in xyz ordering */
        Option<Point3d<X, Y, Z>>, /* min contained point in yzx ordering */
        Option<Point3d<X, Y, Z>>, /* max contained point in yzx ordering */
        Option<Point3d<X, Y, Z>>, /* min contained point in zxy ordering */
        Option<Point3d<X, Y, Z>>, /* max contained point in zxy ordering */
        Option<u8>,               /* own rank */
    ) {
        match self {
            ControlNode::Empty => {
                // Empty tree is a valid tree, nothing to check.
                return (None, None, None, None, None, None, None);
            }
            ControlNode::NonEmpty {
                key,
                rank,
                left,
                right,
                ..
            } => {
                let (
                    left_min_xyz,
                    left_max_xyz,
                    left_min_yzx,
                    left_max_yzx,
                    left_min_zxy,
                    left_max_zxy,
                    left_rank,
                ) = left.do_assert_tree_invariants();

                let (
                    right_min_xyz,
                    right_max_xyz,
                    right_min_yzx,
                    right_max_yzx,
                    right_min_zxy,
                    right_max_zxy,
                    right_rank,
                ) = right.do_assert_tree_invariants();

                if let Some(left_rank) = left_rank {
                    assert!(left_rank < *rank);
                };

                if let Some(right_rank) = right_rank {
                    assert!(right_rank <= *rank);
                };

                if rank % 3 == 0 {
                    if let Some(left_max_zxy) = left_max_zxy.as_ref() {
                        assert_eq!(left_max_zxy.cmp_zxy(key), Ordering::Less);
                    };
                    if let Some(right_min_zxy) = right_min_zxy.as_ref() {
                        assert_eq!(right_min_zxy.cmp_zxy(key), Ordering::Greater);
                    };
                } else if rank % 3 == 1 {
                    if let Some(left_max_yzx) = left_max_yzx.as_ref() {
                        assert_eq!(left_max_yzx.cmp_yzx(key), Ordering::Less);
                    };
                    if let Some(right_min_yzx) = right_min_yzx.as_ref() {
                        assert_eq!(right_min_yzx.cmp_yzx(key), Ordering::Greater);
                    };
                } else {
                    if let Some(left_max_xyz) = left_max_xyz.as_ref() {
                        assert_eq!(left_max_xyz.cmp_xyz(key), Ordering::Less);
                    };
                    if let Some(right_min_xyz) = right_min_xyz.as_ref() {
                        assert_eq!(right_min_xyz.cmp_xyz(key), Ordering::Greater);
                    };
                }

                let mut min_xyz = key.clone();
                if let Some(left_min_xyz) = left_min_xyz {
                    if left_min_xyz.cmp_xyz(&min_xyz) == Ordering::Less {
                        min_xyz = left_min_xyz;
                    }
                };
                if let Some(right_min_xyz) = right_min_xyz {
                    if right_min_xyz.cmp_xyz(&min_xyz) == Ordering::Less {
                        min_xyz = right_min_xyz;
                    }
                };

                let mut max_xyz = key.clone();
                if let Some(left_max_xyz) = left_max_xyz {
                    if left_max_xyz.cmp_xyz(&max_xyz) == Ordering::Greater {
                        max_xyz = left_max_xyz;
                    }
                };
                if let Some(right_max_xyz) = right_max_xyz {
                    if right_max_xyz.cmp_xyz(&max_xyz) == Ordering::Greater {
                        max_xyz = right_max_xyz;
                    }
                };

                let mut min_yzx = key.clone();
                if let Some(left_min_yzx) = left_min_yzx {
                    if left_min_yzx.cmp_yzx(&min_yzx) == Ordering::Less {
                        min_yzx = left_min_yzx;
                    }
                };
                if let Some(right_min_yzx) = right_min_yzx {
                    if right_min_yzx.cmp_yzx(&min_yzx) == Ordering::Less {
                        min_yzx = right_min_yzx;
                    }
                };

                let mut max_yzx = key.clone();
                if let Some(left_max_yzx) = left_max_yzx {
                    if left_max_yzx.cmp_yzx(&max_yzx) == Ordering::Greater {
                        max_yzx = left_max_yzx;
                    }
                };
                if let Some(right_max_yzx) = right_max_yzx {
                    if right_max_yzx.cmp_yzx(&max_yzx) == Ordering::Greater {
                        max_yzx = right_max_yzx;
                    }
                };

                let mut min_zxy = key.clone();
                if let Some(left_min_zxy) = left_min_zxy {
                    if left_min_zxy.cmp_zxy(&min_zxy) == Ordering::Less {
                        min_zxy = left_min_zxy;
                    }
                };
                if let Some(right_min_zxy) = right_min_zxy {
                    if right_min_zxy.cmp_zxy(&min_zxy) == Ordering::Less {
                        min_zxy = right_min_zxy;
                    }
                };

                let mut max_zxy = key.clone();
                if let Some(left_max_zxy) = left_max_zxy {
                    if left_max_zxy.cmp_zxy(&max_zxy) == Ordering::Greater {
                        max_zxy = left_max_zxy;
                    }
                };
                if let Some(right_max_zxy) = right_max_zxy {
                    if right_max_zxy.cmp_zxy(&max_zxy) == Ordering::Greater {
                        max_zxy = right_max_zxy;
                    }
                };

                return (
                    Some(min_xyz),
                    Some(max_xyz),
                    Some(min_yzx),
                    Some(max_yzx),
                    Some(min_zxy),
                    Some(max_zxy),
                    Some(*rank),
                );
            }
        }
    }
}

fn cmp_points_at_rank<X: Dimension, Y: Dimension, Z: Dimension>(
    rank: u8,
    p1: &Point3d<X, Y, Z>,
    p2: &Point3d<X, Y, Z>,
) -> Ordering {
    if rank % 3 == 2 {
        return p1.cmp_xyz(p2);
    } else if rank % 3 == 1 {
        return p1.cmp_yzx(p2);
    } else {
        return p1.cmp_zxy(p2);
    }
}
