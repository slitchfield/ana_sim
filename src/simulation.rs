use std::sync::Weak;

use crate::components::Component;
use crate::node::Node;

#[allow(dead_code)]
#[derive(Default)]
pub struct Simulation {
    node_list: Vec<Weak<Node>>,
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
        let new_component_handle = new_box.get_node_weakref();
        self.node_list.push(new_component_handle);
        self.component_list.push(new_box);
    }

    pub fn get_component_inputs(&mut self) {
        for component in self.component_list.iter_mut() {
            component.pull_in_state();
        }
    }
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
        let a_node = Arc::new(Node::new());
        let b_node = Arc::new(Node::new());
        let resistor = resistor::Resistor::new(Arc::downgrade(&a_node), Arc::downgrade(&b_node));
        let voltage_source = voltage_source::VoltageSource::new(
            Arc::downgrade(&a_node),
            Arc::downgrade(&b_node),
            12.0,
        );
        sim.add_component(resistor);
        sim.add_component(voltage_source);
    }
}
