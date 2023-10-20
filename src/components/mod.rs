pub mod resistor;
pub mod voltage_source;
pub mod wire;

pub trait Component {}

#[allow(unused_imports)]
mod tests {
    use super::{resistor::Resistor, voltage_source::VoltageSource, *};
    use crate::node::Node;
    use std::sync::Arc;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn component_level_test() {
        assert!(true);
    }

    #[test]
    fn voltage_across_resistor_setup() {
        let (pos_id, _pos_node) = Node::new();
        let (gnd_id, _gnd_node) = Node::new();
        let _voltage_source = VoltageSource::new(pos_id, gnd_id, 12.0);
        let _resistor = Resistor::new(pos_id, gnd_id);
    }
}
