use crate::node::*;
use std::sync::Weak;

#[allow(dead_code)]
pub struct Resistor {
    a_node: Weak<Node>,
    b_node: Weak<Node>,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: Weak<Node>, b_node: Weak<Node>) -> Self {
        Resistor { a_node, b_node }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let left_node = Arc::new(Node::new());
        let right_node = Arc::new(Node::new());
        let _ = Resistor::new(Arc::downgrade(&left_node), Arc::downgrade(&right_node));
    }
}
