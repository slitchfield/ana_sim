use crate::components::Component;
use crate::components::Stamp;
use crate::DCComponent;
use std::collections::HashSet;

use nalgebra::base::DMatrix;

#[derive(Debug, PartialEq)]
pub enum NetlistError {
    NoGroundNode,
    NonContiguousNodes { missing: Vec<u64> },
    DuplicateSourceNum { duplicate: u64 },
    NonContiguousSourceNums { missing: Vec<u64> },
    UnknownDependentSource { unknown: u64 },
    ZeroSourceNum,
}

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
    pub a_mat: DMatrix<f64>,
    pub x_mat: DMatrix<f64>,
    pub z_mat: DMatrix<f64>,
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

    pub fn initialize_dc_mna(&mut self) -> Result<(), NetlistError> {
        self.validate_nodes()?;
        self.validate_source_numbering()?;

        // Construct A matrix
        // Dimensions must be N+MxN+M, where N is #nodes and M is #ind v sources
        let n = self.num_nodes();
        self.num_nodes = Some(n);
        let m = self.num_aux_variables();

        self.a_mat = DMatrix::<f64>::from_element(n + m, n + m, 0.0);
        self.x_mat = DMatrix::<f64>::from_element(n + m, 1, 0.0);
        self.z_mat = DMatrix::<f64>::from_element(n + m, 1, 0.0);

        // Construct G matrix from conductances
        let mut g_mat = DMatrix::<f64>::from_element(n, n, 0.0);
        eprintln!(
            "Allocated gmat with shape ({}, {})",
            g_mat.nrows(),
            g_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = component.get_gmat_stamps();
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
            "Allocated bmat with shape ({}, {})",
            b_mat.nrows(),
            b_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = component.get_bmat_stamps();
            for Stamp(r, c, val) in stamps {
                eprintln!("Got bmat stamp: {:?}", Stamp(r, c, val));
                let mut cell = b_mat.view_mut((r - 1, c - 1), (1, 1));
                cell[(0, 0)] += val;
            }
        }
        let mut b_view = self.a_mat.view_mut((0, n), (n, m));
        b_view += b_mat.view_mut((0, 0), (n, m));

        // Construct C matrix from B matrix transpose, handling dependent sources
        // TODO: handle dependent sources
        let mut c_mat = DMatrix::<f64>::from_element(m, n, 0.0);
        eprintln!(
            "Allocated cmat with shape ({}, {})",
            c_mat.nrows(),
            c_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = component.get_cmat_stamps();
            for Stamp(r, c, val) in stamps {
                eprintln!("Got cmat stamp: {:?}", Stamp(r, c, val));
                let mut cell = c_mat.view_mut((r - 1, c - 1), (1, 1));
                cell[(0, 0)] += val;
            }
        }
        let mut c_view = self.a_mat.view_mut((n, 0), (m, n));
        c_view += c_mat.view_mut((0, 0), (m, n));

        // Construct D matrix from dependent sources
        // TODO: handle dependent sources
        let mut d_mat = DMatrix::<f64>::from_element(m, m, 0.0);
        eprintln!(
            "Allocated dmat with shape({}, {})",
            d_mat.nrows(),
            d_mat.ncols()
        );
        for component in &self.component_list {
            let stamps = component.get_dmat_stamps();
            for Stamp(r, c, val) in stamps {
                eprintln!("Got dmat stamp: {:?}", Stamp(r, c, val));
                let mut cell = d_mat.view_mut((r - 1, c - 1), (1, 1));
                cell[(0, 0)] += val;
            }
        }
        let mut d_view = self.a_mat.view_mut((n, n), (m, m));
        d_view += d_mat.view_mut((0, 0), (m, m));

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
                Component::Resistor(_)
                | Component::VCCurrentSource(_)
                | Component::CCCurrentSource(_)
                | Component::CCVoltageSource(_) => {}
            }
        }
        for Stamp(r, c, val) in i_stamps {
            let mut cell = self.z_mat.view_mut((r - 1, c - 1), (1, 1));
            cell[(0, 0)] += val;
        }
        for Stamp(r, c, val) in v_stamps {
            let mut cell = self.z_mat.view_mut((n + r - 1, c - 1), (1, 1));
            cell[(0, 0)] += val;
        }

        self.initialized = true;
        Ok(())
    }

    pub fn solve_dc_mna(&mut self) {
        //TODO: return a result of the solution, encoding whether the solve was initialized or not
        assert!(self.initialized);
        assert!(self.is_linear());

        // Dimensions must be N+MxN+M, where N is #nodes and M is #ind v sources
        let _n = self.num_nodes();
        let _m = self.num_aux_variables();

        eprintln!("A:\n{:.1}", self.a_mat);
        eprintln!("z:\n{:.1}", self.z_mat);

        // Rely on LU factorization to solve these systems
        let lu = self.a_mat.clone().full_piv_lu();
        let result = lu
            .solve(&self.z_mat)
            .expect("Could not solve LU Factorization");
        self.x_mat = result;
        self.x_mat_valid = true;
    }

    fn node_set(&self) -> HashSet<u64> {
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
                Component::VCCurrentSource(depsrc) => {
                    nodeset.insert(depsrc.source_node);
                    nodeset.insert(depsrc.sink_node);
                    nodeset.insert(depsrc.source_sensing_node);
                    nodeset.insert(depsrc.sink_sensing_node);
                }
                Component::CCCurrentSource(depsrc) => {
                    nodeset.insert(depsrc.source_node);
                    nodeset.insert(depsrc.sink_node);
                }
                Component::CCVoltageSource(depsrc) => {
                    nodeset.insert(depsrc.positive_node);
                    nodeset.insert(depsrc.negative_node);
                }
            }
        }
        nodeset
    }

    pub fn num_nodes(&self) -> usize {
        let mut nodeset = self.node_set();
        // MNA A matrix does not include ground node as a node!
        nodeset.remove(&0u64);
        nodeset.len()
    }

    fn validate_nodes(&self) -> Result<(), NetlistError> {
        let mut nodeset = self.node_set();
        if !nodeset.remove(&0u64) {
            return Err(NetlistError::NoGroundNode);
        }

        // n distinct positive node IDs are exactly 1..=n iff none exceeds n
        let max = nodeset.iter().copied().max().unwrap_or(0);
        if max as usize == nodeset.len() {
            return Ok(());
        }
        let missing = (1..=max).filter(|k| !nodeset.contains(k)).collect();
        Err(NetlistError::NonContiguousNodes { missing })
    }

    fn validate_source_numbering(&self) -> Result<(), NetlistError> {
        // Iterate through the voltage sources and CCVS, collecting the `source_num`s

        let mut source_nums: HashSet<u64> = HashSet::new();
        let mut dep_source_nums: Vec<u64> = vec![];

        for component in &self.component_list {
            match component {
                Component::Resistor(_)
                | Component::ICurrentSource(_)
                | Component::VCCurrentSource(_) => {}
                Component::IVoltageSource(vs) => {
                    if !source_nums.insert(vs.source_num) {
                        return Err(NetlistError::DuplicateSourceNum {
                            duplicate: vs.source_num,
                        });
                    }
                }
                Component::CCCurrentSource(depsrc) => {
                    dep_source_nums.push(depsrc.dep_source_num);
                }
                Component::CCVoltageSource(depsrc) => {
                    if !source_nums.insert(depsrc.source_num) {
                        return Err(NetlistError::DuplicateSourceNum {
                            duplicate: depsrc.source_num,
                        });
                    }
                    dep_source_nums.push(depsrc.dep_source_num);
                }
            }
        }

        // Check for dependent source nums in the set
        for depsrc in dep_source_nums {
            if !source_nums.contains(&depsrc) {
                return Err(NetlistError::UnknownDependentSource { unknown: depsrc });
            }
        }

        // Check for a `0` source num value
        if source_nums.contains(&0) {
            return Err(NetlistError::ZeroSourceNum);
        }
        // Check for contiguity
        // m distinct positive source numbers are exactly 1..=m iff none exceeds m
        let max = source_nums.iter().copied().max().unwrap_or(0);
        if max as usize == source_nums.len() {
            return Ok(());
        }
        let missing = (1..=max).filter(|k| !source_nums.contains(k)).collect();
        Err(NetlistError::NonContiguousSourceNums { missing })
    }

    pub fn num_aux_variables(&self) -> usize {
        let mut num_aux_variables = 0usize;
        for component in &self.component_list {
            match component {
                Component::IVoltageSource(_) => {
                    num_aux_variables += 1;
                }
                Component::CCVoltageSource(_) => {
                    num_aux_variables += 1;
                }
                Component::ICurrentSource(_)
                | Component::Resistor(_)
                | Component::VCCurrentSource(_)
                | Component::CCCurrentSource(_) => {}
            }
        }

        num_aux_variables
    }

    pub fn get_node_voltages(&self) -> Option<DMatrix<f64>> {
        match self.num_nodes {
            Some(n) if self.x_mat_valid => Some(self.x_mat.view((0, 0), (n, 1)).into()),
            _ => None,
        }
    }

    pub fn dump_a_mat(&self) {
        if self.initialized {
            for row_num in 0..self.a_mat.nrows() {
                print!("Row {:4}: ", row_num);
                for col_num in 0..self.a_mat.ncols() {
                    print!("\t{:+02.06} |", self.a_mat[(row_num, col_num)]);
                }
                println!();
            }
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

            a_mat: nalgebra::dmatrix![],
            x_mat: nalgebra::dmatrix![],
            z_mat: nalgebra::dmatrix![],
        }
    }
}

#[allow(unused_imports)]
mod tests {
    use super::*;
    use crate::components::cc_current_source;
    use crate::components::cc_voltage_source;
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

        net.initialize_dc_mna().unwrap();

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

        net.initialize_dc_mna().unwrap();

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

        net.initialize_dc_mna().unwrap();

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

        net.initialize_dc_mna().unwrap();

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages.view((0, 0), (1, 1))[(0, 0)], 1.0f64);
        assert_float_relative_eq!(node_voltages.view((1, 0), (1, 1))[(0, 0)], 4.0f64);
    }

    #[test]
    fn current_sources_into_same_node_accumulate() {
        // 1A + 2A into node 1 through 1 ohm to ground -> 3V
        let mut net = Netlist::new();

        let i1 = independent_current_source::ICurrentSource::new(0, 1, 1.0);
        let i2 = independent_current_source::ICurrentSource::new(0, 1, 2.0);
        let r1 = resistor::Resistor::new(1, 0, 1.0);

        net.add_component(Component::ICurrentSource(i1));
        net.add_component(Component::ICurrentSource(i2));
        net.add_component(Component::Resistor(r1));

        net.initialize_dc_mna().unwrap();

        assert_float_relative_eq!(net.z_mat[(0, 0)], 3.0f64);

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages[(0, 0)], 3.0f64);
    }

    #[test]
    fn opposing_current_sources_at_same_node_cancel() {
        // 3A into node 1, 1A out of node 1, through 2 ohm to ground -> 4V
        let mut net = Netlist::new();

        let i1 = independent_current_source::ICurrentSource::new(0, 1, 3.0);
        let i2 = independent_current_source::ICurrentSource::new(1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 0, 2.0);

        net.add_component(Component::ICurrentSource(i1));
        net.add_component(Component::ICurrentSource(i2));
        net.add_component(Component::Resistor(r1));

        net.initialize_dc_mna().unwrap();

        assert_float_relative_eq!(net.z_mat[(0, 0)], 2.0f64);

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages[(0, 0)], 4.0f64);
    }

    #[test]
    fn current_sources_sharing_floating_node_accumulate() {
        // I1: 1A ground -> node 1, I2: 2A node 1 -> node 2, 1 ohm from each node to ground.
        // Net injection: node 1 = 1 - 2 = -1A -> -1V, node 2 = 2A -> 2V
        let mut net = Netlist::new();

        let i1 = independent_current_source::ICurrentSource::new(0, 1, 1.0);
        let i2 = independent_current_source::ICurrentSource::new(1, 2, 2.0);
        let r1 = resistor::Resistor::new(1, 0, 1.0);
        let r2 = resistor::Resistor::new(2, 0, 1.0);

        net.add_component(Component::ICurrentSource(i1));
        net.add_component(Component::ICurrentSource(i2));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));

        net.initialize_dc_mna().unwrap();

        net.solve_dc_mna();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages[(0, 0)], -1.0f64);
        assert_float_relative_eq!(node_voltages[(1, 0)], 2.0f64);
    }

    #[test]
    fn reinitialize_does_not_accumulate_z() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        net.initialize_dc_mna().unwrap();
        net.initialize_dc_mna().unwrap();

        assert_float_relative_eq!(net.z_mat[(0, 0)], 0.0f64);
        assert_float_relative_eq!(net.z_mat[(1, 0)], 1.0f64);
        assert_float_relative_eq!(net.z_mat[(2, 0)], 1.0f64);
    }

    #[test]
    fn repeated_solve_does_not_accumulate_x() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 5.0);
        let r2 = resistor::Resistor::new(0, 2, 10.0);
        let i1 = independent_current_source::ICurrentSource::new(0, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::ICurrentSource(i1));

        net.initialize_dc_mna().unwrap();

        net.solve_dc_mna();
        net.solve_dc_mna();

        assert_float_relative_eq!(net.x_mat[(0, 0)], 1.0f64);
        assert_float_relative_eq!(net.x_mat[(1, 0)], 4.0f64);
        assert_float_relative_eq!(net.x_mat[(2, 0)], 0.6f64);
    }

    #[test]
    fn validate_nodes_rejects_gap() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(3, 0, 1.0)));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::NonContiguousNodes { missing: vec![2] })
        );
        assert!(!net.initialized);
    }

    #[test]
    fn validate_nodes_rejects_numbering_from_two() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(2, 3, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(3, 0, 1.0)));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::NonContiguousNodes { missing: vec![1] })
        );
    }

    #[test]
    fn validate_nodes_reports_all_missing() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(2, 5, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(5, 0, 1.0)));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::NonContiguousNodes {
                missing: vec![1, 3, 4]
            })
        );
    }

    #[test]
    fn validate_nodes_rejects_missing_ground() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 2, 1.0)));

        assert_eq!(net.initialize_dc_mna(), Err(NetlistError::NoGroundNode));
    }

    #[test]
    fn validate_nodes_accepts_contiguous() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(2, 1, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));

        assert_eq!(net.initialize_dc_mna(), Ok(()));
    }

    #[test]
    fn validate_sources_rejects_duplicate_voltage_sources() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let v2 = independent_voltage_source::IVoltageSource::new(1, 2, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(v2));
        net.add_component(Component::Resistor(r1));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::DuplicateSourceNum { duplicate: 1 })
        );
        assert!(!net.initialized);
    }

    #[test]
    fn validate_sources_rejects_duplicate_across_types() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let ccvs = cc_voltage_source::CCVoltageSource::new(1, 1, 0, 0, 2, 0, 1.0);
        let r1 = resistor::Resistor::new(2, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::CCVoltageSource(ccvs));
        net.add_component(Component::Resistor(r1));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::DuplicateSourceNum { duplicate: 1 })
        );
    }

    #[test]
    fn validate_sources_rejects_gap() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let v3 = independent_voltage_source::IVoltageSource::new(3, 2, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(v3));
        net.add_component(Component::Resistor(r1));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::NonContiguousSourceNums { missing: vec![2] })
        );
    }

    #[test]
    fn validate_sources_rejects_zero() {
        let mut net = Netlist::new();

        let v0 = independent_voltage_source::IVoltageSource::new(0, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 0, 1.0);

        net.add_component(Component::IVoltageSource(v0));
        net.add_component(Component::Resistor(r1));

        assert_eq!(net.initialize_dc_mna(), Err(NetlistError::ZeroSourceNum));
    }

    #[test]
    fn validate_sources_rejects_unknown_dependency() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 0, 1.0);
        let cccs = cc_current_source::CCCurrentSource::new(5, 1, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::CCCurrentSource(cccs));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::UnknownDependentSource { unknown: 5 })
        );
    }

    #[test]
    fn validate_sources_accepts_ccvs_controlled_by_ccvs() {
        let mut net = Netlist::new();

        // v1 drives node 1; vsense measures current into r1; ccvs2 is controlled by
        // vsense, and ccvs3 is controlled by ccvs2's branch current
        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let vsense = independent_voltage_source::IVoltageSource::new(2, 1, 2, 0.0);
        let r1 = resistor::Resistor::new(2, 0, 1.0);
        let ccvs2 = cc_voltage_source::CCVoltageSource::new(3, 2, 0, 0, 3, 0, 1.0);
        let r2 = resistor::Resistor::new(3, 0, 1.0);
        let ccvs3 = cc_voltage_source::CCVoltageSource::new(4, 3, 0, 0, 4, 0, 1.0);
        let r3 = resistor::Resistor::new(4, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(vsense));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::CCVoltageSource(ccvs2));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::CCVoltageSource(ccvs3));
        net.add_component(Component::Resistor(r3));

        assert_eq!(net.initialize_dc_mna(), Ok(()));
    }
}
