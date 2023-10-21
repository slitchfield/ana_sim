pub mod independent_current_source;
pub mod independent_voltage_source;
pub mod resistor;

#[derive(Debug)]
pub struct Stamp(pub usize, pub usize, pub f64);

#[allow(dead_code)]
pub enum Component {
    Resistor(resistor::Resistor),
    IVoltageSource(independent_voltage_source::IVoltageSource),
    ICurrentSource(independent_current_source::ICurrentSource),
}

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
