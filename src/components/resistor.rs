use crate::node::*;

#[allow(dead_code)]
pub struct Resistor {
    a_node: Node,
    b_node: Node,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: Node, b_node: Node) -> Self {
        Resistor { a_node, b_node }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let left_node = Node::new();
        let right_node = Node::new();
        let _ = Resistor::new(left_node, right_node);
    }
}
