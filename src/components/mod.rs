pub mod independent_current_source;
pub mod independent_voltage_source;
pub mod resistor;
pub mod vc_current_source;

#[derive(Debug)]
pub struct Stamp(pub usize, pub usize, pub f64);

#[allow(dead_code)]
pub enum Component {
    Resistor(resistor::Resistor),
    IVoltageSource(independent_voltage_source::IVoltageSource),
    ICurrentSource(independent_current_source::ICurrentSource),
    VCCurrentSource(vc_current_source::VCCurrentSource),
}

impl Component {
    pub fn is_linear(&self) -> bool {
        match self {
            Component::Resistor(res) => res.is_linear(),
            Component::IVoltageSource(vs) => vs.is_linear(),
            Component::ICurrentSource(is) => is.is_linear(),
            Component::VCCurrentSource(vccs) => vccs.is_linear(),
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::{independent_voltage_source::IVoltageSource, resistor::Resistor, *};
    use std::sync::Arc;

    #[allow(clippy::assertions_on_constants)]
    #[test]
    fn component_level_test() {
        assert!(true);
    }
}
