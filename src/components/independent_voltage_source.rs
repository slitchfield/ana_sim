#[allow(dead_code)]
#[derive(Debug)]
pub struct IVoltageSource {
    positive_node: u64,
    ground_node: u64,
    voltage: f64,
}

#[allow(dead_code)]
impl IVoltageSource {
    pub fn new(positive_node: u64, ground_node: u64, voltage: f64) -> Self {
        Self {
            positive_node,
            ground_node,
            voltage,
        }
    }
}

use crate::components::Component;
impl Component for IVoltageSource {}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let (pid, _pnode) = Node::new();
        let (gid, _gnode) = Node::new();
        let _ = IVoltageSource::new(pid, gid, 12.0f64);
    }
}
