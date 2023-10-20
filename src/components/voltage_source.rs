use crate::node::Node;

#[allow(dead_code)]
pub struct VoltageSource {
    positive_node: Node,
    ground_node: Node,
    voltage: f64,
}

#[allow(dead_code)]
impl VoltageSource {
    pub fn new(positive_node: Node, ground_node: Node, voltage: f64) -> Self {
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

    #[test]
    fn creation() {
        let pos_node = Node::new();
        let gnd_node = Node::new();
        let _ = VoltageSource::new(pos_node, gnd_node, 12.0f64);
    }
}
