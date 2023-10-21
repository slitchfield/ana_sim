#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Resistor {
    pub a_node: u64,
    pub b_node: u64,
    pub resistance: f64,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: u64, b_node: u64, resistance: f64) -> Self {
        Self {
            a_node,
            b_node,
            resistance,
            //..Default::default()
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
        let (lid, _lnode) = Node::new();
        let (rid, _rnode) = Node::new();
        let _ = Resistor::new(lid, rid, 1.0);
    }
}
