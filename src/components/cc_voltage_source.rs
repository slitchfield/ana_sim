use super::Stamp;
use crate::DCComponent;

#[derive(Debug)]
pub struct CCVoltageSource {
    pub source_num: u64,
    pub dep_source_num: u64,
    pub positive_node: u64,
    pub negative_node: u64,
    gain: f64,
}

impl CCVoltageSource {
    pub fn new(
        source_num: u64,
        dep_source_num: u64,
        positive_node: u64,
        negative_node: u64,
        gain: f64,
    ) -> Self {
        Self {
            source_num,
            dep_source_num,
            positive_node,
            negative_node,
            gain,
        }
    }

    pub fn is_linear(&self) -> bool {
        true
    }
}

impl DCComponent for CCVoltageSource {
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
        let mut retvec: Vec<Stamp> = vec![];
        if self.positive_node != 0 {
            retvec.push(Stamp(self.positive_node as _, self.source_num as _, 1.0));
        }
        if self.negative_node != 0 {
            retvec.push(Stamp(self.negative_node as _, self.source_num as _, -1.0));
        }
        retvec
    }

    fn get_cmat_stamps(&self) -> Vec<Stamp> {
        let mut retvec: Vec<Stamp> = vec![];

        if self.positive_node != 0 {
            retvec.push(Stamp(self.source_num as _, self.positive_node as _, 1.0));
        }
        if self.negative_node != 0 {
            retvec.push(Stamp(self.source_num as _, self.negative_node as _, -1.0))
        }
        retvec
    }

    fn get_dmat_stamps(&self) -> Vec<Stamp> {
        vec![Stamp(
            self.source_num as _,
            self.dep_source_num as _,
            self.gain,
        )]
    }

    fn get_ivec_stamps(&self) -> Vec<Stamp> {
        vec![]
    }

    fn get_evec_stamps(&self) -> Vec<Stamp> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::*;
    use crate::netlist::Netlist;
    use assert_float_eq::*;

    #[test]
    fn creation() {
        let _ = CCVoltageSource::new(0, 1, 0, 1, 12.0f64);
    }

    #[test]
    fn basic_function_grounded() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 2.0);
        let v2 = independent_voltage_source::IVoltageSource::new(2, 2, 1, 0.0);
        let r1 = resistor::Resistor::new(2, 3, 1.0);
        let r2 = resistor::Resistor::new(0, 3, 1.0);
        let ccvs = cc_voltage_source::CCVoltageSource::new(3, 2, 3, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(v2));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::CCVoltageSource(ccvs));

        net.initialize_dc_mna().unwrap();

        net.solve_dc_mna().unwrap();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((1 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((2 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((3 - 1, 0), (1, 1))[(0, 0)], 1.0f64);
    }

    #[test]
    fn basic_function_ungrounded() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 2.0);
        // 0V source senses the current from node 1 to node 2
        let vsense = independent_voltage_source::IVoltageSource::new(2, 1, 2, 0.0);
        let r1 = resistor::Resistor::new(2, 0, 1.0);
        // Floating output across nodes 3 and 4; V3 - V4 = -gain * j(vsense)
        let ccvs = cc_voltage_source::CCVoltageSource::new(3, 2, 3, 4, 1.0);
        let r3 = resistor::Resistor::new(3, 0, 1.0);
        let r4 = resistor::Resistor::new(4, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(vsense));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r3));
        net.add_component(Component::Resistor(r4));
        net.add_component(Component::CCVoltageSource(ccvs));

        net.initialize_dc_mna().unwrap();

        net.solve_dc_mna().unwrap();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((1 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((2 - 1, 0), (1, 1))[(0, 0)], 2.0f64);
        assert_float_relative_eq!(node_voltages.view((3 - 1, 0), (1, 1))[(0, 0)], -1.0f64);
        assert_float_relative_eq!(node_voltages.view((4 - 1, 0), (1, 1))[(0, 0)], 1.0f64);
    }
}
