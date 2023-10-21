use crate::components::Component;
use crate::components::Stamp;
use std::collections::HashSet;

use nalgebra::base::DMatrix;

#[allow(dead_code)]
pub struct Netlist {
    component_list: Vec<Component>,

    a_mat: Box<DMatrix<f64>>,
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
        let n = self.num_nodes();
        let m = self.num_iv_sources();

        let Netlist {
            component_list: _,
            a_mat,
        } = self;

        let new_a_mat = a_mat.to_owned().resize(n + m, n + m, 0.0);
        self.a_mat = Box::new(new_a_mat);

        for component in &self.component_list {
            let stamps = match component {
                Component::Resistor(res) => res.get_stamps(),
                _ => {
                    vec![]
                }
            };
            for Stamp(r, c, val) in stamps {
                let mut cell = self.a_mat.view_mut((r, c), (1, 1));
                cell[(0, 0)] += val;
            }
        }
    }

    pub fn num_nodes(&self) -> usize {
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
                Component::ICurrentSource(is) => {
                    nodeset.insert(is.source_node);
                    nodeset.insert(is.sink_node);
                }
            }
        }
        // MNA A matrix does not include ground node as a node!
        nodeset.remove(&0u64);
        nodeset.len()
    }

    pub fn num_iv_sources(&self) -> usize {
        let mut num_iv_sources = 0usize;
        for component in &self.component_list {
            if let Component::IVoltageSource(_) = component {
                num_iv_sources += 1;
            }
        }

        num_iv_sources
    }
}

impl Default for Netlist {
    fn default() -> Self {
        Self {
            component_list: vec![],
            a_mat: Box::new(nalgebra::dmatrix![]),
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::independent_current_source;
    use crate::components::independent_voltage_source;
    use crate::components::resistor;
    use crate::components::Component;
    use crate::node::Node;
    use assert_float_eq::*;
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

        let resistor = resistor::Resistor::new(pos_id, gnd_id, 1.0);
        let voltage_source = independent_voltage_source::IVoltageSource::new(pos_id, gnd_id, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }

    #[test]
    fn simple_vs_resistor_test() {
        let mut net = Netlist::new();
        let (pos_id, _pos_node) = Node::new();
        let (gnd_id, _gnd_node) = Node::new();

        let resistor = resistor::Resistor::new(pos_id, gnd_id, 1.0);
        let voltage_source = independent_voltage_source::IVoltageSource::new(pos_id, gnd_id, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }

    #[test]
    fn init_matrix_dimensions() {
        let mut net = Netlist::new();

        let resistor = resistor::Resistor::new(1, 0, 1.0);
        let voltage_source = independent_voltage_source::IVoltageSource::new(1, 0, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));

        // a_mat is uninitialized and thus 0x0
        assert!(net.a_mat.ncols() == 0);
        assert!(net.a_mat.nrows() == 0);

        net.initialize_dc_mna();

        // a_mat should now be n+m x n+m, i.e. (1 node + 1 indep vsource)
        assert!(net.a_mat.ncols() == 2);
        assert!(net.a_mat.nrows() == 2);
    }

    #[test]
    fn init_matrix_values() {
        // Based on example from S3.1.5 of http://qucs.github.io/docs/technical/technical.pdf
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        assert!(net.a_mat.ncols() == 0);
        assert!(net.a_mat.nrows() == 0);
        net.initialize_dc_mna();

        // a_mat should now be n+m x n+m, i.e. (2 node + 1 indep vsource)
        assert!(net.a_mat.ncols() == 3);
        assert!(net.a_mat.nrows() == 3);

        assert_float_relative_eq!(net.a_mat.view((1, 1), (1, 1))[(0, 0)], 0.2f64);
        assert_float_relative_eq!(net.a_mat.view((1, 2), (1, 1))[(0, 0)], -0.2f64);
        assert_float_relative_eq!(net.a_mat.view((2, 1), (1, 1))[(0, 0)], -0.2f64);
        assert_float_relative_eq!(net.a_mat.view((2, 2), (1, 1))[(0, 0)], 0.3f64);
    }
}
