use super::Stamp;
use crate::DCComponent;

#[allow(dead_code)]
#[derive(Debug)]
pub struct ICurrentSource {
    pub source_node: u64,
    pub sink_node: u64,
    current: f64,
}

#[allow(dead_code)]
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

    fn get_zmat_stamps(&self) -> Vec<Stamp> {
        // The z matrix is 1×(M+N) (N is the number of nodes, and M is the number of independent
        //   voltage sources) and:
        //    • the i matrix is 1×N and contains the sum of the currents through the passive elements into
        //      the corresponding node (either zero, or the sum of independent current sources
        let mut retvec: Vec<Stamp> = vec![];
        if self.source_node != 0 {
            retvec.push(Stamp(self.source_node as _, 1, -self.current));
        }
        if self.sink_node != 0 {
            retvec.push(Stamp(self.sink_node as _, 1, self.current));
        }
        retvec
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let _ = ICurrentSource::new(0, 1, 12.0f64);
    }
}
