use crate::fm_constr_mgr::FMConstrMgr;
use crate::hypergraph::Hypergraph;

/// Binary Constraint Manager
///
/// Selects the partition to move to (the one with smaller weight).
/// Ported from C++ `FMBiConstrMgr` in `FMBiConstrMgr.hpp`.
pub struct FMBiConstrMgr<Gnl: Hypergraph>(pub FMConstrMgr<Gnl>);

impl<Gnl: Hypergraph> FMBiConstrMgr<Gnl> {
    pub fn new(hyprgraph: Gnl, bal_tol: f64) -> Self {
        FMBiConstrMgr(FMConstrMgr::new(hyprgraph, bal_tol))
    }

    pub fn with_num_parts(hyprgraph: Gnl, bal_tol: f64, _num_parts: u8) -> Self {
        FMBiConstrMgr(FMConstrMgr::new(hyprgraph, bal_tol))
    }

    pub fn select_togo(&self) -> u8 {
        self.0.select_togo()
    }
}

impl<Gnl: Hypergraph> std::ops::Deref for FMBiConstrMgr<Gnl> {
    type Target = FMConstrMgr<Gnl>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Gnl: Hypergraph> std::ops::DerefMut for FMBiConstrMgr<Gnl> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
