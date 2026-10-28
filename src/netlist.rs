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
    UninitializedSolve,
    NonlinearSolve,
    SingularMatrix,
    ShortedOutput { index: usize },
    ShortedControl { index: usize },
}

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
        self.validate_terminals()?;

        // A is (N+M)×(N+M); x and z are (N+M)×1
        let n = self.num_nodes();
        self.num_nodes = Some(n);
        let m = self.num_aux_variables();

        self.a_mat = DMatrix::<f64>::from_element(n + m, n + m, 0.0);
        self.x_mat = DMatrix::<f64>::from_element(n + m, 1, 0.0);
        self.z_mat = DMatrix::<f64>::from_element(n + m, 1, 0.0);

        // A = [G B; C D], z = [i; e]
        //   G (N×N) conductances, B (N×M) / C (M×N) voltage-source connections,
        //   D (M×M) dependent-source coupling
        //   i (N×1) net independent current into each node, e (M×1) independent source voltages
        // N is the number of non-ground nodes and M the number of auxiliary current unknowns
        for component in &self.component_list {
            add_stamps(&mut self.a_mat, 0, 0, component.get_gmat_stamps());
            add_stamps(&mut self.a_mat, 0, n, component.get_bmat_stamps());
            add_stamps(&mut self.a_mat, n, 0, component.get_cmat_stamps());
            add_stamps(&mut self.a_mat, n, n, component.get_dmat_stamps());
            add_stamps(&mut self.z_mat, 0, 0, component.get_ivec_stamps());
            add_stamps(&mut self.z_mat, n, 0, component.get_evec_stamps());
        }

        self.initialized = true;
        Ok(())
    }

    pub fn solve_dc_mna(&mut self) -> Result<(), NetlistError> {
        if !self.initialized {
            return Err(NetlistError::UninitializedSolve);
        }
        if !self.is_linear() {
            return Err(NetlistError::NonlinearSolve);
        }

        // Rely on LU factorization to solve these systems
        let lu = self.a_mat.clone().full_piv_lu();
        let result = lu.solve(&self.z_mat).ok_or(NetlistError::SingularMatrix)?;
        self.x_mat = result;
        self.x_mat_valid = true;
        Ok(())
    }

    fn node_set(&self) -> HashSet<u64> {
        self.component_list
            .iter()
            .flat_map(|component| component.nodes())
            .collect()
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

    fn validate_terminals(&self) -> Result<(), NetlistError> {
        // Sensing and output nodes may legally overlap (e.g. a VCCS sensing its own output acts
        // as a conductance), but a shorted output or control pair is always a mistake
        for (index, component) in self.component_list.iter().enumerate() {
            match component {
                Component::Resistor(_)
                | Component::IVoltageSource(_)
                | Component::ICurrentSource(_) => {}
                Component::VCCurrentSource(depsrc) => {
                    if depsrc.source_node == depsrc.sink_node {
                        return Err(NetlistError::ShortedOutput { index });
                    }
                    if depsrc.source_sensing_node == depsrc.sink_sensing_node {
                        return Err(NetlistError::ShortedControl { index });
                    }
                }
                Component::CCCurrentSource(depsrc) => {
                    if depsrc.source_node == depsrc.sink_node {
                        return Err(NetlistError::ShortedOutput { index });
                    }
                }
                Component::CCVoltageSource(depsrc) => {
                    if depsrc.positive_node == depsrc.negative_node {
                        return Err(NetlistError::ShortedOutput { index });
                    }
                }
            }
        }
        Ok(())
    }

    pub fn num_aux_variables(&self) -> usize {
        self.component_list
            .iter()
            .map(|component| component.num_aux_variables())
            .sum()
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

// Stamps are 1-indexed within their block; the offsets place that block within `target`
fn add_stamps(target: &mut DMatrix<f64>, row_offset: usize, col_offset: usize, stamps: Vec<Stamp>) {
    for Stamp(r, c, val) in stamps {
        target[(row_offset + r - 1, col_offset + c - 1)] += val;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::cc_current_source;
    use crate::components::cc_voltage_source;
    use crate::components::independent_current_source;
    use crate::components::independent_voltage_source;
    use crate::components::resistor;
    use crate::components::vc_current_source;
    use crate::components::Component;
    use assert_float_eq::*;

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

        net.solve_dc_mna().unwrap();

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

        net.solve_dc_mna().unwrap();

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

        net.solve_dc_mna().unwrap();

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

        net.solve_dc_mna().unwrap();

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

        net.solve_dc_mna().unwrap();

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

        net.solve_dc_mna().unwrap();
        net.solve_dc_mna().unwrap();

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
        let ccvs = cc_voltage_source::CCVoltageSource::new(1, 1, 2, 0, 1.0);
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
        let ccvs2 = cc_voltage_source::CCVoltageSource::new(3, 2, 3, 0, 1.0);
        let r2 = resistor::Resistor::new(3, 0, 1.0);
        let ccvs3 = cc_voltage_source::CCVoltageSource::new(4, 3, 4, 0, 1.0);
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

    #[test]
    fn solve_without_initialize_fails() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));

        assert_eq!(net.solve_dc_mna(), Err(NetlistError::UninitializedSolve));
        assert!(net.get_node_voltages().is_none());
    }

    #[test]
    fn solve_parallel_voltage_sources_is_singular() {
        // Passes node/source validation, but the two source rows are identical
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let v2 = independent_voltage_source::IVoltageSource::new(2, 1, 0, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::IVoltageSource(v2));

        net.initialize_dc_mna().unwrap();

        assert_eq!(net.solve_dc_mna(), Err(NetlistError::SingularMatrix));
        assert!(net.get_node_voltages().is_none());
    }

    #[test]
    fn solve_floating_subcircuit_is_singular() {
        // Nodes 2 and 3 are connected only to each other, so their voltages are undetermined
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(2, 3, 1.0)));

        net.initialize_dc_mna().unwrap();

        assert_eq!(net.solve_dc_mna(), Err(NetlistError::SingularMatrix));
    }

    #[test]
    fn validate_terminals_rejects_shorted_vccs_output() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(2, 0, 1.0)));
        net.add_component(Component::VCCurrentSource(
            vc_current_source::VCCurrentSource::new(1, 0, 2, 2, 1.0),
        ));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::ShortedOutput { index: 2 })
        );
        assert!(!net.initialized);
    }

    #[test]
    fn validate_terminals_rejects_shorted_vccs_control() {
        let mut net = Netlist::new();

        net.add_component(Component::Resistor(resistor::Resistor::new(1, 0, 1.0)));
        net.add_component(Component::Resistor(resistor::Resistor::new(2, 0, 1.0)));
        net.add_component(Component::VCCurrentSource(
            vc_current_source::VCCurrentSource::new(1, 1, 2, 0, 1.0),
        ));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::ShortedControl { index: 2 })
        );
    }

    #[test]
    fn validate_terminals_rejects_shorted_cccs_output() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 0, 1.0);
        let cccs = cc_current_source::CCCurrentSource::new(1, 1, 1, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::CCCurrentSource(cccs));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::ShortedOutput { index: 2 })
        );
    }

    #[test]
    fn validate_terminals_rejects_shorted_ccvs_output() {
        let mut net = Netlist::new();

        let v1 = independent_voltage_source::IVoltageSource::new(1, 1, 0, 1.0);
        let r1 = resistor::Resistor::new(1, 2, 1.0);
        let r2 = resistor::Resistor::new(2, 0, 1.0);
        let ccvs = cc_voltage_source::CCVoltageSource::new(2, 1, 2, 2, 1.0);

        net.add_component(Component::IVoltageSource(v1));
        net.add_component(Component::Resistor(r1));
        net.add_component(Component::Resistor(r2));
        net.add_component(Component::CCVoltageSource(ccvs));

        assert_eq!(
            net.initialize_dc_mna(),
            Err(NetlistError::ShortedOutput { index: 3 })
        );
    }

    #[test]
    fn validate_terminals_accepts_vccs_sensing_own_output() {
        // A VCCS sensing its own output nodes behaves as a 0.5 ohm conductance: V1 = 1A / 2S
        let mut net = Netlist::new();

        let i1 = independent_current_source::ICurrentSource::new(0, 1, 1.0);
        let vccs = vc_current_source::VCCurrentSource::new(1, 0, 1, 0, 2.0);

        net.add_component(Component::ICurrentSource(i1));
        net.add_component(Component::VCCurrentSource(vccs));

        net.initialize_dc_mna().unwrap();
        net.solve_dc_mna().unwrap();

        let node_voltages = net
            .get_node_voltages()
            .expect("voltages should be valid here");
        assert_float_relative_eq!(node_voltages[(0, 0)], 0.5f64);
    }
}
