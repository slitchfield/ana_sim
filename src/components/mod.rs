pub mod cc_current_source;
pub mod cc_voltage_source;
pub mod independent_current_source;
pub mod independent_voltage_source;
pub mod resistor;
pub mod vc_current_source;

use crate::DCComponent;

#[derive(Debug)]
pub struct Stamp(pub usize, pub usize, pub f64);

pub enum Component {
    Resistor(resistor::Resistor),
    IVoltageSource(independent_voltage_source::IVoltageSource),
    ICurrentSource(independent_current_source::ICurrentSource),
    VCCurrentSource(vc_current_source::VCCurrentSource),
    CCCurrentSource(cc_current_source::CCCurrentSource),
    CCVoltageSource(cc_voltage_source::CCVoltageSource),
}

impl Component {
    pub fn is_linear(&self) -> bool {
        match self {
            Component::Resistor(res) => res.is_linear(),
            Component::IVoltageSource(vs) => vs.is_linear(),
            Component::ICurrentSource(is) => is.is_linear(),
            Component::VCCurrentSource(vccs) => vccs.is_linear(),
            Component::CCCurrentSource(cccs) => cccs.is_linear(),
            Component::CCVoltageSource(ccvs) => ccvs.is_linear(),
        }
    }
}

impl DCComponent for Component {
    fn nodes(&self) -> Vec<u64> {
        match self {
            Component::Resistor(res) => res.nodes(),
            Component::IVoltageSource(vs) => vs.nodes(),
            Component::ICurrentSource(is) => is.nodes(),
            Component::VCCurrentSource(vccs) => vccs.nodes(),
            Component::CCCurrentSource(cccs) => cccs.nodes(),
            Component::CCVoltageSource(ccvs) => ccvs.nodes(),
        }
    }
    fn num_aux_variables(&self) -> usize {
        match self {
            Component::Resistor(res) => res.num_aux_variables(),
            Component::IVoltageSource(vs) => vs.num_aux_variables(),
            Component::ICurrentSource(is) => is.num_aux_variables(),
            Component::VCCurrentSource(vccs) => vccs.num_aux_variables(),
            Component::CCCurrentSource(cccs) => cccs.num_aux_variables(),
            Component::CCVoltageSource(ccvs) => ccvs.num_aux_variables(),
        }
    }
    fn get_gmat_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_gmat_stamps(),
            Component::IVoltageSource(vs) => vs.get_gmat_stamps(),
            Component::ICurrentSource(is) => is.get_gmat_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_gmat_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_gmat_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_gmat_stamps(),
        }
    }
    fn get_bmat_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_bmat_stamps(),
            Component::IVoltageSource(vs) => vs.get_bmat_stamps(),
            Component::ICurrentSource(is) => is.get_bmat_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_bmat_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_bmat_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_bmat_stamps(),
        }
    }
    fn get_cmat_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_cmat_stamps(),
            Component::IVoltageSource(vs) => vs.get_cmat_stamps(),
            Component::ICurrentSource(is) => is.get_cmat_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_cmat_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_cmat_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_cmat_stamps(),
        }
    }
    fn get_dmat_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_dmat_stamps(),
            Component::IVoltageSource(vs) => vs.get_dmat_stamps(),
            Component::ICurrentSource(is) => is.get_dmat_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_dmat_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_dmat_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_dmat_stamps(),
        }
    }
    fn get_ivec_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_ivec_stamps(),
            Component::IVoltageSource(vs) => vs.get_ivec_stamps(),
            Component::ICurrentSource(is) => is.get_ivec_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_ivec_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_ivec_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_ivec_stamps(),
        }
    }
    fn get_evec_stamps(&self) -> Vec<Stamp> {
        match self {
            Component::Resistor(res) => res.get_evec_stamps(),
            Component::IVoltageSource(vs) => vs.get_evec_stamps(),
            Component::ICurrentSource(is) => is.get_evec_stamps(),
            Component::VCCurrentSource(vccs) => vccs.get_evec_stamps(),
            Component::CCCurrentSource(cccs) => cccs.get_evec_stamps(),
            Component::CCVoltageSource(ccvs) => ccvs.get_evec_stamps(),
        }
    }
}
