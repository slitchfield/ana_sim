use super::Stamp;
use crate::DCComponent;

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

    pub fn is_linear(&self) -> bool {
        true
    }
}

impl DCComponent for IVoltageSource {
    fn get_gmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_bmat_stamps(&self) -> Vec<Stamp> {
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
            ret_vec.push(Stamp(self.negative_node as _, self.source_num as _, -1.0));
        }

        ret_vec
    }

    fn get_cmat_stamps(&self) -> Vec<Stamp> {
        // The C matrix is an N×M matrix with only 0, 1 and -1 elements. Each location in the matrix
        // corresponds to a particular voltage source (first dimension) or a node (second dimension). If the
        // positive terminal of the ith voltage source is connected to node k, then the element (i, k) in the
        // B matrix is a 1. If the negative terminal of the ith voltage source is connected to node k, then
        //the element (i, k) in the C matrix is a -1. Otherwise, elements of the C matrix are zero.
        let mut ret_vec: Vec<Stamp> = vec![];
        if self.positive_node != 0 {
            ret_vec.push(Stamp(self.source_num as _, self.positive_node as _, 1.0));
        }
        if self.negative_node != 0 {
            ret_vec.push(Stamp(self.source_num as _, self.negative_node as _, -1.0));
        }

        ret_vec
    }

    fn get_dmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_zmat_stamps(&self) -> Vec<Stamp> {
        // The z matrix is 1×(M+N) (N is the number of nodes, and M is the number of independent
        //    voltage sources) and:
        //    • the i matrix is 1×N and contains the sum of the currents through the passive elements into
        //    the corresponding node (either zero, or the sum of independent current sources)
        //    • the e matrix is 1×M and holds the values of the independent voltage sources
        vec![Stamp(self.source_num as _, 1, self.voltage)]
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let _ = IVoltageSource::new(1, 1, 0, 12.0f64);
    }
}
