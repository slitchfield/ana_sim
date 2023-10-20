pub mod resistor;
pub mod voltage_source;

use crate::node::Node;
use std::sync::Weak;

pub trait Component {
    fn pull_in_state(&mut self);
    fn get_node_weakref(&self) -> Weak<Node>;
}

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
        let pos_node = Arc::new(Node::new());
        let gnd_node = Arc::new(Node::new());
        let _voltage_source =
            VoltageSource::new(Arc::downgrade(&pos_node), Arc::downgrade(&gnd_node), 12.0);
        let _resistor = Resistor::new(Arc::downgrade(&pos_node), Arc::downgrade(&gnd_node));
    }
}
