use super::Stamp;
use crate::DCComponent;

#[derive(Debug)]
pub struct ICurrentSource {
    pub source_node: u64,
    pub sink_node: u64,
    current: f64,
}

impl ICurrentSource {
    pub fn new(source_node: u64, sink_node: u64, current: f64) -> Self {
        Self {
            source_node,
            sink_node,
            current,
        }
    }

    pub fn is_linear(&self) -> bool {
        true
    }
}

impl DCComponent for ICurrentSource {
    fn nodes(&self) -> Vec<u64> {
        vec![self.source_node, self.sink_node]
    }

    fn num_aux_variables(&self) -> usize {
        0
    }

    fn get_gmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_bmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_cmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_dmat_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_ivec_stamps(&self) -> Vec<Stamp> {
        // Row is the node number within the i block of z: the net current injected into the node
        let mut retvec: Vec<Stamp> = vec![];
        if self.source_node != 0 {
            retvec.push(Stamp(self.source_node as _, 1, -self.current));
        }
        if self.sink_node != 0 {
            retvec.push(Stamp(self.sink_node as _, 1, self.current));
        }
        retvec
    }

    fn get_evec_stamps(&self) -> Vec<Stamp> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation() {
        let _ = ICurrentSource::new(0, 1, 12.0f64);
    }
}
