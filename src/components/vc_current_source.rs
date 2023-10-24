use super::Stamp;

#[allow(dead_code)]
#[derive(Debug)]
pub struct VCCurrentSource {
    pub source_sensing_node: u64,
    pub sink_sensing_node: u64,
    pub source_node: u64,
    pub sink_node: u64,
    gain: f64,
}

#[allow(dead_code)]
impl VCCurrentSource {
    pub fn new(
        source_sensing_node: u64,
        sink_sensing_node: u64,
        source_node: u64,
        sink_node: u64,
        gain: f64,
    ) -> Self {
        // TODO: check that sensing/output nodes don't overlap illegally
        Self {
            source_sensing_node,
            sink_sensing_node,
            source_node,
            sink_node,
            gain,
        }
    }

    pub fn is_linear(&self) -> bool {
        true
    }

    pub fn get_gmat_stamps(&self) -> Vec<Stamp> {
        let mut retvec: Vec<Stamp> = vec![];
        if self.source_node != 0 && self.source_sensing_node != 0 {
            retvec.push(Stamp(
                self.source_node as _,
                self.source_sensing_node as _,
                self.gain,
            ));
        }
        if self.sink_node != 0 && self.source_sensing_node != 0 {
            retvec.push(Stamp(
                self.sink_node as _,
                self.source_sensing_node as _,
                -self.gain,
            ));
        }
        if self.source_node != 0 && self.sink_sensing_node != 0 {
            retvec.push(Stamp(
                self.source_node as _,
                self.sink_sensing_node as _,
                -self.gain,
            ));
        }
        if self.sink_node != 0 && self.sink_sensing_node != 0 {
            retvec.push(Stamp(
                self.sink_node as _,
                self.sink_sensing_node as _,
                self.gain,
            ));
        }
        retvec
    }

    pub fn get_zmat_stamps(&self) -> Vec<Stamp> {
        todo!();
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::netlist::Netlist;
    use assert_float_eq::*;
    use std::sync::Arc;

    #[test]
    fn creation() {
        let _ = VCCurrentSource::new(0, 1, 0, 1, 12.0f64);
    }

    #[test]
    fn basic_function_grounded() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 2.0);
        let r1 = resistor::Resistor::new(1, 2, 1.0);
        let r2 = resistor::Resistor::new(0, 2, 1.0);
        let vccs = vc_current_source::VCCurrentSource::new(2, 0, 3, 0, -1.0);
        let r3 = resistor::Resistor::new(3, 0, 2.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::Resistor(r3));
        net.add_component(Component::VCCurrentSource(vccs));

        net.initialize_dc_mna();

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((1 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((2 - 1, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(node_voltages.view((3 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
    }

    #[test]
    fn basic_function_ungrounded() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 2.0);
        let r1 = resistor::Resistor::new(1, 2, 1.0);
        let r2 = resistor::Resistor::new(0, 2, 1.0);
        let vccs = vc_current_source::VCCurrentSource::new(2, 0, 4, 3, 1.0);
        let r3 = resistor::Resistor::new(3, 0, 1.0);
        let r4 = resistor::Resistor::new(4, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::Resistor(r3));
        net.add_component(Component::Resistor(r4));
        net.add_component(Component::VCCurrentSource(vccs));

        net.initialize_dc_mna();

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((1 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((2 - 1, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(node_voltages.view((3 - 1, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(node_voltages.view((4 - 1, 0), (1, 1))[(0, 0)], -1.0f64);
    }
}
