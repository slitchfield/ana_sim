pub mod independent_voltage_source;
pub mod resistor;

pub trait Component {}

#[allow(unused_imports)]
mod tests {
    use super::{independent_voltage_source::IVoltageSource, resistor::Resistor, *};
    use crate::node::Node;
    use std::sync::Arc;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn component_level_test() {
        assert!(true);
    }
}
