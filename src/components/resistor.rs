use crate::components::Stamp;

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

    pub fn get_stamps(&self) -> Vec<Stamp> {
        let mut ret_vec: Vec<Stamp> = vec![];

        // Calculate diagonal elements as 1 / resistance
        // Ignore if node is the ground node!
        if self.a_node != 0 {
            ret_vec.push(Stamp(
                self.a_node as usize,
                self.a_node as usize,
                1.0f64 / self.resistance,
            ));
        }
        if self.b_node != 0 {
            ret_vec.push(Stamp(
                self.b_node as usize,
                self.b_node as usize,
                1.0f64 / self.resistance,
            ));
        }

        if self.a_node != 0 && self.b_node != 0 {
            ret_vec.push(Stamp(
                self.a_node as usize,
                self.b_node as usize,
                -1.0f64 / self.resistance,
            ));
            ret_vec.push(Stamp(
                self.b_node as usize,
                self.a_node as usize,
                -1.0f64 / self.resistance,
            ));
        }

        ret_vec
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
