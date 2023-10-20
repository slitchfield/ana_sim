use std::collections::HashMap;

use crate::components::Component;
use crate::node::Node;

#[allow(dead_code)]
#[derive(Default)]
pub struct Simulation {
    node_list: HashMap<u64, Node>,
    component_list: Vec<Box<dyn Component>>,
    time_step: f64,
    duration: f64,
}

#[allow(dead_code)]
impl Simulation {
    pub fn new() -> Self {
        Simulation {
            ..Default::default()
        }
    }

    pub fn add_component(&mut self, new_component: impl Component + 'static) {
        let new_box = Box::new(new_component);
        self.component_list.push(new_box);
    }

    pub fn spread_voltages(&mut self) {}
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::resistor;
    use crate::components::voltage_source;
    use crate::components::Component;
    use crate::node::Node;
    use std::sync::Arc;

    #[test]
    fn simulation_creation() {
        let _ = Simulation::new();
    }

    #[test]
    fn adding_components() {
        let mut sim = Simulation::new();
        let (pos_id, pos_node) = Node::new();
        let (gnd_id, mut gnd_node) = Node::new();
        gnd_node.make_ground();
        sim.node_list.insert(pos_id, pos_node);
        sim.node_list.insert(gnd_id, gnd_node);

        let resistor = resistor::Resistor::new(pos_id, gnd_id);
        let voltage_source = voltage_source::VoltageSource::new(pos_id, gnd_id, 12.0);
        sim.add_component(resistor);
        sim.add_component(voltage_source);
    }
}
