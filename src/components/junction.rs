#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Junction {
    nodes: Vec<u64>,
    inouts: Vec<bool>,
}

#[allow(dead_code)]
impl Junction {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn add_node(&mut self, node_id: u64, is_input: bool) {
        self.nodes.push(node_id);
        self.inouts.push(is_input);
    }
}

use crate::components::Component;
impl Component for Junction {}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let input_node = Node::new();
        let output_node_a = Node::new();
        let output_node_b = Node::new();
        let junk = Junction::new();
        junk.add_node(input_node, true);
        junk.add_node(output_node_a, false);
        junk.add_node(output_node_b, false);
    }
}
