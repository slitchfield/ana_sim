pub mod components;
pub mod netlist;

use crate::components::Stamp;
pub trait DCComponent {
    fn get_gmat_stamps(&self) -> Vec<Stamp>;
    fn get_bmat_stamps(&self) -> Vec<Stamp>;
    fn get_cmat_stamps(&self) -> Vec<Stamp>;
    fn get_dmat_stamps(&self) -> Vec<Stamp>;
    fn get_zmat_stamps(&self) -> Vec<Stamp>;
}
