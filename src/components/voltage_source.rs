use crate::node::Node;
use std::sync::Arc;
use std::sync::Weak;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VoltageSource {
    own_node: Arc<Node>,
    positive_node: Weak<Node>,
    ground_node: Weak<Node>,
    voltage: f64,
}

#[allow(dead_code)]
impl VoltageSource {
    pub fn new(positive_node: Weak<Node>, ground_node: Weak<Node>, voltage: f64) -> Self {
        let own_node = Arc::new(Node::new());
        VoltageSource {
            own_node,
            positive_node,
            ground_node,
            voltage,
        }
    }
}

use crate::components::Component;
impl Component for VoltageSource {
    fn get_node_weakref(&self) -> Weak<Node> {
        Arc::downgrade(&self.own_node)
    }

    fn pull_in_state(&self) {}
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let pos_node = Arc::new(Node::new());
        let gnd_node = Arc::new(Node::new());
        let _ = VoltageSource::new(
            Arc::downgrade(&pos_node),
            Arc::downgrade(&gnd_node),
            12.0f64,
        );
    }
}
