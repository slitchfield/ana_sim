use std::sync::Arc;
use std::sync::Weak;

use crate::components::Component;
use crate::node::Node;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct Simulation {
    node_list: Vec<Weak<Node>>,
    component_list: Vec<Arc<Component>>,
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

    pub fn add_component(&mut self, new_component: Component) {
        let new_component_handle: Weak<Node> = match &new_component {
            Component::Resistor(res) => res.get_node_weakref(),
            Component::VoltageSource(vs) => vs.get_node_weakref(),
        };
        let new_arc = Arc::new(new_component);
        self.node_list.push(new_component_handle);
        self.component_list.push(new_arc);
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::resistor;
    use crate::components::voltage_source;
    use crate::components::Component;
    use crate::node::Node;

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
        sim.add_component(Component::Resistor(resistor));
        sim.add_component(Component::VoltageSource(voltage_source));
    }
}
