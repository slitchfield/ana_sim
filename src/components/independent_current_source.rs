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
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let _ = ICurrentSource::new(0, 1, 12.0f64);
    }
}
