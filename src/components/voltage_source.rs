#[allow(dead_code)]
#[derive(Debug)]
pub struct VoltageSource {
    positive_node: u64,
    ground_node: u64,
    voltage: f64,
}

#[allow(dead_code)]
impl VoltageSource {
    pub fn new(positive_node: u64, ground_node: u64, voltage: f64) -> Self {
        Self {
            positive_node,
            ground_node,
            voltage,
        }
    }
}

use crate::components::Component;
impl Component for VoltageSource {}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let pos_node = Node::new();
        let gnd_node = Node::new();
        let _ = VoltageSource::new(pos_node.id, gnd_node.id, 12.0f64);
    }
}
