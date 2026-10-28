use std::collections::HashMap;

use crate::{
    common::NodeAction,
    interpreter::{
        BehaviorTree,
        stack::{Call, Transform},
    },
};

type StackDepth = usize;

/// Responsible for taking popped nodes and creating transforms
#[derive(Default)]
pub struct TransformStack {
    list: Vec<Transform>,
    stack_depth: StackDepth,
    applied_transforms: HashMap<StackDepth, Vec<NodeAction>>,
}

impl TransformStack {
    pub fn take_frame(&mut self, tree: &BehaviorTree, frame: Call) {
        let Ok(t) = tree.get_node_action(frame.node_index()).try_into() else {
            return;
        };

        self.list.push(t);
        self.stack_depth = self.list.len() - 1;
    }
}
