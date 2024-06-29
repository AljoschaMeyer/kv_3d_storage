#![no_main]
use libfuzzer_sys::fuzz_target;

use std::collections::HashMap;

use kv_3d_storage::*;
use kv_3d_storage_fuzz::*;

fuzz_target!(|data: HashMap<
    Point3d<U8FixedWidth, U8FixedWidth, U8FixedWidth>,
    (u8 /* value */, u8 /* rank */),
>| {
    let tree: ControlNode<_, _, _, _, usize> = ControlNode::from_iter(
        data.clone()
            .drain()
            .map(|(point, (value, rank))| (point, value, rank)),
    );

    tree.assert_tree_invariants();

    match tree {
        ControlNode::Empty => {
            assert_eq!(data.len(), 0);
        }
        ControlNode::NonEmpty { count, summary, .. } => {
            assert_eq!(count, data.len());
            assert_eq!(summary, data.len());
        }
    }
});
