use crate::components::Component;
use std::collections::HashSet;

#[allow(dead_code)]
#[derive(Default)]
pub struct Netlist {
    component_list: Vec<Component>,
}

#[allow(dead_code)]
impl Netlist {
    pub fn new() -> Self {
        Netlist {
            ..Default::default()
        }
    }

    pub fn add_component(&mut self, new_component: Component) {
        self.component_list.push(new_component);
    }

    pub fn initialize_dc_mna(&mut self) {
        // Construct A matrix
        // Dimensions must be N+MxN+M, where N is #nodes and M is #ind v sources
        let _n = self.num_nodes();
    }

    pub fn num_nodes(&self) -> u64 {
        let mut nodeset: HashSet<u64> = HashSet::<u64>::new();

        for component in &self.component_list {
            match component {
                Component::Resistor(res) => {
                    nodeset.insert(res.a_node);
                    nodeset.insert(res.b_node);
                }
                Component::IVoltageSource(vs) => {
                    nodeset.insert(vs.positive_node);
                    nodeset.insert(vs.ground_node);
                }
            }
        }

        0
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::independent_voltage_source;
    use crate::components::resistor;
    use crate::components::Component;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn simulation_creation() {
        let _ = Netlist::new();
    }

    #[test]
    fn adding_components() {
        let mut net = Netlist::new();
        let (pos_id, _pos_node) = Node::new();
        let (gnd_id, mut gnd_node) = Node::new();
        gnd_node.make_ground();

        let resistor = resistor::Resistor::new(pos_id, gnd_id);
        let voltage_source = independent_voltage_source::IVoltageSource::new(pos_id, gnd_id, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }

    #[test]
    fn simple_vs_resistor_test() {
        let mut net = Netlist::new();
        let (pos_id, _pos_node) = Node::new();
        let (gnd_id, _gnd_node) = Node::new();

        let resistor = resistor::Resistor::new(pos_id, gnd_id);
        let voltage_source = independent_voltage_source::IVoltageSource::new(pos_id, gnd_id, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }
}
