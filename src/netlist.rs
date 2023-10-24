use crate::components::Component;
use crate::components::Stamp;
use std::collections::HashSet;

use nalgebra::base::DMatrix;

#[allow(dead_code)]
pub struct Netlist {
    component_list: Vec<Component>,
    initialized: bool,
    x_mat_valid: bool,
    num_nodes: Option<usize>,
    // TODO: evaluate possibilities for Option(x_mat) instead
    //   Pros: cleaner representation, more idiomatic
    //   Cons: more frequent allocation?

    // TODO: change to sparse matrix representation
    //   nalgebra-sparse will likely help
    a_mat: Box<DMatrix<f64>>,
    x_mat: Box<DMatrix<f64>>,
    z_mat: Box<DMatrix<f64>>,
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

    pub fn is_linear(&self) -> bool {
        self.component_list.iter().all(|c| c.is_linear())
    }

    pub fn initialize_dc_mna(&mut self) {
        // Construct A matrix
        // Dimensions must be N+MxN+M, where N is #nodes and M is #ind v sources
        let n = self.num_nodes();
        self.num_nodes = Some(n);
        let m = self.num_iv_sources();

        self.a_mat = Box::new(DMatrix::<f64>::from_element(n + m, n + m, 0.0));
        self.x_mat = Box::new(DMatrix::<f64>::from_element(n + m, 1, 0.0));
        self.z_mat = Box::new(DMatrix::<f64>::from_element(n + m, 1, 0.0));

        // Construct G matrix from conductances
        let mut g_mat = DMatrix::<f64>::from_element(n, n, 0.0);
        eprintln!(
            "Allocated gmat with shape ({}, {})",
            g_mat.nrows(),
            g_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = match component {
                Component::Resistor(res) => res.get_gmat_stamps(),
                Component::ICurrentSource(_is) => {
                    vec![]
                }
                Component::VCCurrentSource(vccs) => vccs.get_gmat_stamps(),
                Component::IVoltageSource(_vs) => {
                    vec![]
                }
            };
            for Stamp(r, c, val) in stamps {
                eprintln!("Got gmat stamp: {:?}", Stamp(r, c, val));
                let mut cell = g_mat.view_mut((r - 1, c - 1), (1, 1));
                cell[(0, 0)] += val;
            }
        }
        let mut g_view = self.a_mat.view_mut((0, 0), (n, n));
        g_view += g_mat;

        // Construct B matrix from independent V sources
        let mut b_mat = DMatrix::<f64>::from_element(n, m, 0.0);
        eprintln!(
            "Allocated gmat with shape ({}, {})",
            b_mat.nrows(),
            b_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = match component {
                Component::IVoltageSource(vs) => vs.get_bmat_stamps(),
                Component::Resistor(_) => vec![],
                Component::ICurrentSource(_) => vec![],
                Component::VCCurrentSource(_) => vec![],
            };

            for Stamp(r, c, val) in stamps {
                eprintln!("Got bmat stamp: {:?}", Stamp(r, c, val));
                let mut cell = b_mat.view_mut((r - 1, c - 1), (1, 1));
                cell[(0, 0)] = val;
            }
        }
        let mut b_view = self.a_mat.view_mut((0, n), (n, m));
        b_view += b_mat;

        // Construct C matrix from B matrix transpose, handling dependent sources
        // TODO: handle dependent sources
        let c_mat = b_view.transpose();
        let mut c_view = self.a_mat.view_mut((n, 0), (m, n));
        c_view += c_mat;

        // Construct D matrix from dependent sources
        // TODO: handle dependent sources

        // Construct Z matrix from independent sources
        // The z matrix holds our independent voltage and current sources and will be developed as the
        // combination of 2 smaller matrices i and e. It is quite easy to formulate.
        // z = [i; e]
        // The z matrix is 1×(M+N) (N is the number of nodes, and M is the number of independent
        // voltage sources) and:
        // • the i matrix is 1×N and contains the sum of the currents through the passive elements into
        //   the corresponding node (either zero, or the sum of independent current sources)
        // • the e matrix is 1×M and holds the values of the independent voltage source
        let mut v_stamps: Vec<Stamp> = vec![];
        let mut i_stamps: Vec<Stamp> = vec![];
        for component in &self.component_list {
            match component {
                Component::IVoltageSource(vs) => {
                    v_stamps.append(&mut vs.get_zmat_stamps());
                }
                Component::ICurrentSource(is) => {
                    i_stamps.append(&mut is.get_zmat_stamps());
                }
                Component::Resistor(_) => {}
                Component::VCCurrentSource(_) => {}
            }
        }
        for Stamp(r, c, val) in i_stamps {
            let mut cell = self.z_mat.view_mut((r - 1, c - 1), (1, 1));
            cell[(0, 0)] = val;
        }
        for Stamp(r, c, val) in v_stamps {
            let mut cell = self.z_mat.view_mut((n + r - 1, c - 1), (1, 1));
            cell[(0, 0)] = val;
        }

        self.initialized = true;
    }

    pub fn solve_dc_mna(&mut self) {
        //TODO: return a result of the solution, encoding whether the solve was initialized or not
        assert!(self.initialized);
        assert!(self.is_linear());

        // Dimensions must be N+MxN+M, where N is #nodes and M is #ind v sources
        let n = self.num_nodes();
        let m = self.num_iv_sources();

        // TODO: change from inversion to LU/Gauss elimination
        let a_inv = self
            .a_mat
            .clone()
            .try_inverse()
            .expect("Could not invert the A Matrix!");
        let a_inv_view = a_inv.view((0, 0), (n + m, n + m));
        let z_mat_view = self.z_mat.view((0, 0), (n + m, 1));
        let x_mat = a_inv_view * z_mat_view;

        let mut x_mat_view = self.x_mat.view_mut((0, 0), (n + m, 1));
        x_mat_view += x_mat;
        self.x_mat_valid = true;
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
                    nodeset.insert(vs.negative_node);
                }
                Component::ICurrentSource(is) => {
                    nodeset.insert(is.source_node);
                    nodeset.insert(is.sink_node);
                }
                Component::VCCurrentSource(vccs) => {
                    nodeset.insert(vccs.source_node);
                    nodeset.insert(vccs.sink_node);
                    nodeset.insert(vccs.source_sensing_node);
                    nodeset.insert(vccs.sink_sensing_node);
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

    pub fn get_node_voltages(&self) -> Option<DMatrix<f64>> {
        if self.x_mat_valid && self.num_nodes.is_some() {
            Some(
                self.x_mat
                    .view(
                        (0, 0),
                        (
                            self.num_nodes
                                .expect("Checked that num_nodes is some, but is none"),
                            1,
                        ),
                    )
                    .into(),
            )
        } else {
            None
        }
    }
}

impl Default for Netlist {
    fn default() -> Self {
        Self {
            component_list: vec![],
            initialized: false,
            num_nodes: None,
            x_mat_valid: false,

            a_mat: Box::new(nalgebra::dmatrix![]),
            x_mat: Box::new(nalgebra::dmatrix![]),
            z_mat: Box::new(nalgebra::dmatrix![]),
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
    use assert_float_eq::*;
    use std::sync::Arc;

    #[test]
    fn simulation_creation() {
        let _ = Netlist::new();
    }

    #[test]
    fn adding_components() {
        let mut net = Netlist::new();

        let resistor = resistor::Resistor::new(1, 0, 1.0);
        let voltage_source = independent_voltage_source::IVoltageSource::new(1, 1, 0, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }

    #[test]
    fn simple_vs_resistor_test() {
        let mut net = Netlist::new();

        let resistor = resistor::Resistor::new(1, 0, 1.0);
        let voltage_source = independent_voltage_source::IVoltageSource::new(1, 1, 0, 12.0);
        net.add_component(Component::Resistor(resistor));
        net.add_component(Component::IVoltageSource(voltage_source));
    }

    #[test]
    fn init_matrix_dimensions() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        // a_mat is uninitialized and thus 0x0
        assert!(net.a_mat.ncols() == 0);
        assert!(net.a_mat.nrows() == 0);
        assert!(net.z_mat.ncols() == 0);
        assert!(net.z_mat.nrows() == 0);

        net.initialize_dc_mna();

        // a_mat should now be n+m x n+m, i.e. (2 node + 1 indep vsource)
        assert!(net.a_mat.ncols() == 3);
        assert!(net.a_mat.nrows() == 3);
        assert!(net.z_mat.ncols() == 1);
        assert!(net.z_mat.nrows() == 3);
    }

    #[test]
    fn init_matrix_values() {
        // Based on example from S3.1.5 of http://qucs.github.io/docs/technical/technical.pdf
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        net.initialize_dc_mna();

        assert_float_relative_eq!(net.a_mat.view((0, 0), (1, 1))[(0, 0)], 0.2f64);
        assert_float_relative_eq!(net.a_mat.view((0, 1), (1, 1))[(0, 0)], -0.2f64);
        assert_float_relative_eq!(net.a_mat.view((1, 0), (1, 1))[(0, 0)], -0.2f64);
        assert_float_relative_eq!(net.a_mat.view((1, 1), (1, 1))[(0, 0)], 0.3f64);
        assert_float_relative_eq!(net.a_mat.view((0, 2), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(net.a_mat.view((2, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(net.z_mat.view((0, 0), (1, 1))[(0, 0)], 0.0f64);
        assert_float_relative_eq!(net.z_mat.view((1, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(net.z_mat.view((2, 0), (1, 1))[(0, 0)], 1.0f64);
    }

    #[test]
    fn dc_mna_solve() {
        // Based on example from S3.1.5 of http://qucs.github.io/docs/technical/technical.pdf
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        net.initialize_dc_mna();

        net.solve_dc_mna();

        assert_float_relative_eq!(net.x_mat.view((0, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(net.x_mat.view((1, 0), (1, 1))[(0, 0)], 4.0f64);
        assert_float_relative_eq!(net.x_mat.view((2, 0), (1, 1))[(0, 0)], 0.6f64);
    }

    #[test]
    fn get_node_voltages() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        net.initialize_dc_mna();

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((0, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(node_voltages.view((1, 0), (1, 1))[(0, 0)], 4.0f64);
    }
}
