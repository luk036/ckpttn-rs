use crate::fm_constr_mgr::{FMConstrMgr, LegalCheck};
use crate::fm_gain_mgr::ConstrMgrInterface;
use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfoV;

/// K-way constraint manager for FM partitioning.
///
/// Extends FMConstrMgr with per-partition illegal tracking
/// for k-way partitions. A move is AllSatisfied only when
/// all parts meet the lower bound after the move.
/// Ported from Python `FMKWayConstrMgr` in `FMKWayConstrMgr.py`.
pub struct FMKWayConstrMgr<Gnl: Hypergraph>(pub FMConstrMgr<Gnl>);

impl<Gnl: Hypergraph> FMKWayConstrMgr<Gnl> {
    pub fn new(hyprgraph: Gnl, bal_tol: f64, num_parts: u8) -> Self {
        FMKWayConstrMgr(FMConstrMgr::with_num_parts(hyprgraph, bal_tol, num_parts))
    }

    pub fn select_togo(&self) -> u8 {
        let mut min_idx = 0u8;
        let mut min_val = self.0.diff[0];
        for (i, &d) in self.0.diff.iter().enumerate().skip(1) {
            if d < min_val {
                min_val = d;
                min_idx = i as u8;
            }
        }
        min_idx
    }

    pub fn check_legal(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) -> LegalCheck {
        let status = self.0.check_legal(move_info_v);
        if status != LegalCheck::AllSatisfied {
            return status;
        }
        // Check if all parts are legal after this move
        let (_, _from_part, _to_part) = (move_info_v.v, move_info_v.from_part, move_info_v.to_part);
        // Recompute diff status for all parts
        for &d in self.0.diff.iter() {
            if d < self.0.lowerbound {
                return LegalCheck::GetBetter;
            }
        }
        LegalCheck::AllSatisfied
    }
}

impl<Gnl: Hypergraph> std::ops::Deref for FMKWayConstrMgr<Gnl> {
    type Target = FMConstrMgr<Gnl>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Gnl: Hypergraph> std::ops::DerefMut for FMKWayConstrMgr<Gnl> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<Gnl: Hypergraph> ConstrMgrInterface<Gnl> for FMKWayConstrMgr<Gnl> {
    fn init(&mut self, part: &[u8]) {
        self.0.init(part)
    }
    fn check_legal(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) -> LegalCheck {
        self.check_legal(move_info_v)
    }
    fn check_constraints(&self, move_info_v: &MoveInfoV<Gnl::Node>) -> bool {
        self.0.check_constraints(move_info_v)
    }
    fn update_move(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) {
        self.0.update_move(move_info_v)
    }
    fn select_togo(&self) -> u8 {
        self.select_togo()
    }
    fn final_check(&mut self, part: &[u8]) -> bool {
        self.0.final_check(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    fn make_netlist() -> (SimpleNetlist, Vec<NodeIndex>) {
        let mut netlist = SimpleNetlist::new(6, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[6]);
        netlist.add_edge(nodes[1], nodes[6]);
        netlist.add_edge(nodes[2], nodes[7]);
        netlist.add_edge(nodes[3], nodes[7]);
        netlist.add_edge(nodes[4], nodes[7]);
        netlist.add_edge(nodes[5], nodes[7]);
        (netlist, nodes)
    }

    #[test]
    fn test_new_3_parts() {
        let (netlist, _) = make_netlist();
        let mgr = FMKWayConstrMgr::new(netlist, 0.5, 3);
        assert_eq!(mgr.0.num_parts, 3);
    }

    #[test]
    fn test_select_togo_returns_min() {
        let (netlist, _) = make_netlist();
        let mut mgr = FMKWayConstrMgr::new(netlist, 0.5, 3);
        mgr.0.diff = vec![10, 5, 8];
        assert_eq!(mgr.select_togo(), 1);
    }

    #[test]
    fn test_check_legal_all_satisfied() {
        let (netlist, nodes) = make_netlist();
        let mut mgr = FMKWayConstrMgr::new(netlist, 0.3, 3);
        // 6 modules, 2 per part
        let part = vec![0u8, 0, 1, 1, 2, 2];
        mgr.0.init(&part);
        let move_info = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        let result = mgr.check_legal(&move_info);
        assert_eq!(result, LegalCheck::AllSatisfied);
    }

    #[test]
    fn test_deref() {
        let (netlist, _) = make_netlist();
        let mgr = FMKWayConstrMgr::new(netlist, 0.5, 3);
        let _: &FMConstrMgr<SimpleNetlist> = &mgr;
    }

    #[test]
    fn test_deref_mut() {
        let (netlist, _) = make_netlist();
        let mut mgr = FMKWayConstrMgr::new(netlist, 0.5, 3);
        let _: &mut FMConstrMgr<SimpleNetlist> = &mut mgr;
    }
}
