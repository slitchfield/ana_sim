#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct Resistor {
    a_node: u64,
    b_node: u64,

    a_v: f64,
    a_c: f64,
    b_v: f64,
    b_c: f64,
}

#[allow(dead_code)]
impl Resistor {
    pub fn new(a_node: u64, b_node: u64) -> Self {
        Self {
            a_node,
            b_node,
            ..Default::default()
        }
    }
}

use crate::components::Component;
impl Component for Resistor {}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let (lid, _lnode) = Node::new();
        let (rid, _rnode) = Node::new();
        let _ = Resistor::new(lid, rid);
    }
}
