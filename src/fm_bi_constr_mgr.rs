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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;

    #[test]
    fn test_new() {
        let netlist = SimpleNetlist::new(4, 2);
        let mgr = FMBiConstrMgr::new(netlist, 0.5);
        assert_eq!(mgr.0.num_parts, 2);
    }

    #[test]
    fn test_with_num_parts() {
        let netlist = SimpleNetlist::new(4, 2);
        let mgr = FMBiConstrMgr::with_num_parts(netlist, 0.5, 4);
        assert_eq!(mgr.0.num_parts, 2);
    }

    #[test]
    fn test_select_togo() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMBiConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        mgr.0.init(&part);
        let togo = mgr.select_togo();
        assert!(togo == 0 || togo == 1);
    }

    #[test]
    fn test_deref() {
        let netlist = SimpleNetlist::new(4, 2);
        let mgr = FMBiConstrMgr::new(netlist, 0.5);
        let _: &FMConstrMgr<SimpleNetlist> = &*mgr;
    }

    #[test]
    fn test_deref_mut() {
        let netlist = SimpleNetlist::new(4, 2);
        let mut mgr = FMBiConstrMgr::new(netlist, 0.5);
        let _: &mut FMConstrMgr<SimpleNetlist> = &mut *mgr;
    }
}
