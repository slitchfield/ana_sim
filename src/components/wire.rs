#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Wire {
    a_node: u64,
    b_node: u64,
}

#[allow(dead_code)]
impl Wire {
    pub fn new(a_node: u64, b_node: u64) -> Self {
        Self {
            a_node,
            b_node,
            //..Default::default()
        }
    }
}

use crate::components::Component;
impl Component for Wire {}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let left_node = Node::new();
        let right_node = Node::new();
        let _ = Wire::new(left_node.id, right_node.id);
    }
}
