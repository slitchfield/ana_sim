use crate::node::*;
use std::sync::Arc;
use std::sync::Weak;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Resistor {
    own_node: Arc<Node>,
    a_node: Weak<Node>,
    b_node: Weak<Node>,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: Weak<Node>, b_node: Weak<Node>) -> Self {
        let own_node = Arc::new(Node::new());
        Resistor {
            own_node,
            a_node,
            b_node,
        }
    }

    pub fn get_node_weakref(&self) -> Weak<Node> {
        Arc::downgrade(&self.own_node)
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
