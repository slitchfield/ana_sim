use super::Stamp;

#[allow(dead_code)]
#[derive(Debug)]
pub struct IVoltageSource {
    pub source_num: u64,
    pub positive_node: u64,
    pub negative_node: u64,
    voltage: f64,
}

#[allow(dead_code)]
impl IVoltageSource {
    pub fn new(source_num: u64, positive_node: u64, negative_node: u64, voltage: f64) -> Self {
        Self {
            source_num,
            positive_node,
            negative_node,
            voltage,
        }
    }

    pub fn get_bmat_stamps(&self) -> Vec<Stamp> {
        // The B matrix is an N×M matrix with only 0, 1 and -1 elements. Each location in the matrix
        // corresponds to a particular voltage source (first dimension) or a node (second dimension). If the
        // positive terminal of the ith voltage source is connected to node k, then the element (k,i) in the
        // B matrix is a 1. If the negative terminal of the ith voltage source is connected to node k, then
        //the element (k,i) in the B matrix is a -1. Otherwise, elements of the B matrix are zero.
        let mut ret_vec: Vec<Stamp> = vec![];
        if self.positive_node != 0 {
            ret_vec.push(Stamp(self.positive_node as _, self.source_num as _, 1.0));
        }
        if self.negative_node != 0 {
            ret_vec.push(Stamp(self.negative_node as _, self.source_num as _, 1.0));
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
        let (pid, _pnode) = Node::new();
        let (gid, _gnode) = Node::new();
        let _ = IVoltageSource::new(1, pid, gid, 12.0f64);
    }
}
