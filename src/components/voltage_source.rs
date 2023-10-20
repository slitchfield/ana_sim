use crate::node::Node;
use std::sync::Weak;

#[allow(dead_code)]
pub struct VoltageSource {
    positive_node: Weak<Node>,
    ground_node: Weak<Node>,
    voltage: f64,
}

#[allow(dead_code)]
impl VoltageSource {
    pub fn new(positive_node: Weak<Node>, ground_node: Weak<Node>, voltage: f64) -> Self {
        VoltageSource {
            positive_node,
            ground_node,
            voltage,
        }
    }
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
