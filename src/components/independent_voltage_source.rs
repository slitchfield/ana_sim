use super::Stamp;
use crate::DCComponent;

#[derive(Debug)]
pub struct IVoltageSource {
    pub source_num: u64,
    pub positive_node: u64,
    pub negative_node: u64,
    voltage: f64,
}

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
    fn nodes(&self) -> Vec<u64> {
        vec![self.positive_node, self.negative_node]
    }

    fn num_aux_variables(&self) -> usize {
        1
    }

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
        // The C matrix is an M×N matrix with only 0, 1 and -1 elements. Each location in the matrix
        // corresponds to a particular voltage source (first dimension) or a node (second dimension). If the
        // positive terminal of the ith voltage source is connected to node k, then the element (i, k) in the
        // C matrix is a 1. If the negative terminal of the ith voltage source is connected to node k, then
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

    fn get_ivec_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_evec_stamps(&self) -> Vec<Stamp> {
        // Row is the source number within the e block of z
        vec![Stamp(self.source_num as _, 1, self.voltage)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let _ = IVoltageSource::new(1, 1, 0, 12.0f64);
    }
}
