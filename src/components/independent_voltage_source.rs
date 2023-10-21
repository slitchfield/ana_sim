#[allow(dead_code)]
#[derive(Debug)]
pub struct IVoltageSource {
    pub positive_node: u64,
    pub ground_node: u64,
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
