use crate::node::*;
use std::sync::Arc;
use std::sync::Weak;

#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Resistor {
    own_node: Arc<Node>,
    a_node: Weak<Node>,
    b_node: Weak<Node>,

    a_v: f64,
    a_c: f64,
    b_v: f64,
    b_c: f64,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: Weak<Node>, b_node: Weak<Node>) -> Self {
        let own_node = Arc::new(Node::new());
        Resistor {
            own_node,
            a_node,
            b_node,
            ..Default::default()
        }
    }

    pub fn get_node_weakref(&self) -> Weak<Node> {
        Arc::downgrade(&self.own_node)
    }
}

use crate::components::Component;
impl Component for Resistor {
    fn get_node_weakref(&self) -> Weak<Node> {
        Arc::downgrade(&self.own_node)
    }

    fn pull_in_state(&mut self) {
        (self.a_v, self.a_c) = self
            .a_node
            .upgrade()
            .expect("A_node was dropped!")
            .get_state();
        (self.b_v, self.b_c) = self
            .b_node
            .upgrade()
            .expect("B_node was dropped!")
            .get_state();
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
