pub fn add(left: usize, right: usize) -> usize {
    left + right
}

mod components;
mod netlist;

use crate::components::Stamp;
pub trait DCComponent {
    fn get_gmat_stamps(&self) -> Vec<Stamp>;
    fn get_bmat_stamps(&self) -> Vec<Stamp>;
    fn get_cmat_stamps(&self) -> Vec<Stamp>;
    fn get_dmat_stamps(&self) -> Vec<Stamp>;
    fn get_zmat_stamps(&self) -> Vec<Stamp>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
