pub mod components;
pub mod netlist;

use crate::components::Stamp;
pub trait DCComponent {
    /// Nodes this component connects to, including ground (0)
    fn nodes(&self) -> Vec<u64>;
    /// Number of auxiliary current unknowns this component adds to the MNA system
    fn num_aux_variables(&self) -> usize;

    fn get_gmat_stamps(&self) -> Vec<Stamp>;
    fn get_bmat_stamps(&self) -> Vec<Stamp>;
    fn get_cmat_stamps(&self) -> Vec<Stamp>;
    fn get_dmat_stamps(&self) -> Vec<Stamp>;
    /// Right-hand side stamps for the i block of z; rows are node numbers
    fn get_ivec_stamps(&self) -> Vec<Stamp>;
    /// Right-hand side stamps for the e block of z; rows are source numbers
    fn get_evec_stamps(&self) -> Vec<Stamp>;
}
