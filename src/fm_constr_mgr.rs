use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfoV;

/// Result of a legality check for a proposed move.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegalCheck {
    NotSatisfied,
    GetBetter,
    AllSatisfied,
}

/// Fiduccia-Mattheyses Partition Constraint Manager
///
/// Ported from C++ `FMConstrMgr` in `FMConstrMgr.hpp`/`FMConstrMgr.cpp`.
#[allow(dead_code)]
pub struct FMConstrMgr<Gnl: Hypergraph> {
    hyprgraph: Gnl,
    bal_tol: f64,
    total_weight: u32,
    weight_cache: u32,
    pub diff: Vec<u32>,
    pub lowerbound: u32,
    pub num_parts: u8,
}

impl<Gnl: Hypergraph> FMConstrMgr<Gnl> {
    /// Creates a new FMConstrMgr with 2 partitions.
    pub fn new(hyprgraph: Gnl, bal_tol: f64) -> Self {
        Self::with_num_parts(hyprgraph, bal_tol, 2)
    }

    /// Creates a new FMConstrMgr with the specified number of partitions.
    pub fn with_num_parts(hyprgraph: Gnl, bal_tol: f64, num_parts: u8) -> Self {
        let mut total_weight = 0u32;
        for v in hyprgraph.modules() {
            total_weight += hyprgraph.get_module_weight(v);
        }
        let totalweight_k = (total_weight as f64) * (2.0 / num_parts as f64);
        let lowerbound = (totalweight_k * bal_tol).round() as u32;

        FMConstrMgr {
            hyprgraph,
            bal_tol,
            total_weight,
            weight_cache: 0,
            diff: vec![0; num_parts as usize],
            lowerbound,
            num_parts,
        }
    }

    /// Initializes the diff vector based on the given partition.
    pub fn init(&mut self, part: &[u8]) {
        for d in &mut self.diff {
            *d = 0;
        }
        for (i, module) in self.hyprgraph.modules().enumerate() {
            let p = part[i] as usize;
            if p < self.diff.len() {
                self.diff[p] += self.hyprgraph.get_module_weight(module);
            }
        }
    }

    /// Checks the legality of a move.
    pub fn check_legal(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) -> LegalCheck {
        self.weight_cache = self.hyprgraph.get_module_weight(move_info_v.v);
        let diff_from = self.diff[move_info_v.from_part as usize];
        if diff_from < self.lowerbound + self.weight_cache {
            return LegalCheck::NotSatisfied;
        }
        let diff_to = self.diff[move_info_v.to_part as usize];
        if diff_to + self.weight_cache < self.lowerbound {
            return LegalCheck::GetBetter;
        }
        LegalCheck::AllSatisfied
    }

    /// Checks if the move satisfies balance constraints.
    pub fn check_constraints(&self, move_info_v: &MoveInfoV<Gnl::Node>) -> bool {
        let weight = self.hyprgraph.get_module_weight(move_info_v.v);
        let diff_from = self.diff[move_info_v.from_part as usize];
        diff_from >= self.lowerbound + weight
    }

    /// Updates internal state after a move.
    pub fn update_move(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) {
        self.diff[move_info_v.to_part as usize] += self.weight_cache;
        self.diff[move_info_v.from_part as usize] -= self.weight_cache;
    }

    /// Performs a final legality check on a partition.
    pub fn final_check(&mut self, part: &[u8]) -> bool {
        self.init(part);
        for &d in &self.diff {
            if d < self.lowerbound {
                return false;
            }
        }
        true
    }

    /// Returns the partition with smaller weight (for binary partitioning).
    pub fn select_togo(&self) -> u8 {
        if self.diff[0] < self.diff[1] {
            0
        } else {
            1
        }
    }
}

use crate::fm_gain_mgr::ConstrMgrInterface;

impl<Gnl: Hypergraph> ConstrMgrInterface<Gnl> for FMConstrMgr<Gnl> {
    fn init(&mut self, part: &[u8]) {
        self.init(part)
    }
    fn check_legal(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) -> LegalCheck {
        self.check_legal(move_info_v)
    }
    fn check_constraints(&self, move_info_v: &MoveInfoV<Gnl::Node>) -> bool {
        self.check_constraints(move_info_v)
    }
    fn update_move(&mut self, move_info_v: &MoveInfoV<Gnl::Node>) {
        self.update_move(move_info_v)
    }
    fn select_togo(&self) -> u8 {
        self.select_togo()
    }
    fn final_check(&mut self, part: &[u8]) -> bool {
        self.final_check(part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    #[test]
    fn test_new_with_2_parts() {
        let netlist = SimpleNetlist::new(4, 2);
        let mgr = FMConstrMgr::new(netlist, 0.5);
        assert_eq!(mgr.num_parts, 2);
    }

    #[test]
    fn test_init_and_check() {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);

        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 1]; // 3 modules in part 0, 1 in part 1
        mgr.init(&part);

        assert_eq!(mgr.diff[0], 3);
        assert_eq!(mgr.diff[1], 1);

        // Moving from 0 to 1: diff_from=3 >= lowerbound(2)+weight(1)=3 → ok
        let move_info = MoveInfoV {
            v: nodes[2], // weight 1, in part 0
            from_part: 0,
            to_part: 1,
        };
        let check = mgr.check_legal(&move_info);
        assert_eq!(check, LegalCheck::AllSatisfied);
    }

    #[test]
    fn test_update_move() {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);

        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);

        let move_info = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        let _ = mgr.check_legal(&move_info);
        mgr.update_move(&move_info);

        assert_eq!(mgr.diff[0], 1);
        assert_eq!(mgr.diff[1], 3);
    }

    #[test]
    fn test_final_check() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        assert!(mgr.final_check(&part));
    }

    #[test]
    fn test_select_togo() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);
        let togo = mgr.select_togo();
        assert!(togo == 0 || togo == 1);
    }

    #[test]
    fn test_check_legal_not_satisfied() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        let check = mgr.check_legal(&move_info);
        assert_eq!(check, LegalCheck::NotSatisfied);
    }

    #[test]
    fn test_check_legal_get_better() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        mgr.init(&part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        let check = mgr.check_legal(&move_info);
        assert_eq!(check, LegalCheck::GetBetter);
    }

    #[test]
    fn test_check_constraints_true() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        mgr.init(&part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert!(mgr.check_constraints(&move_info));
    }

    #[test]
    fn test_check_constraints_false() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert!(!mgr.check_constraints(&move_info));
    }

    #[test]
    fn test_final_check_false() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        assert!(!mgr.final_check(&part));
    }

    #[test]
    fn test_select_togo_known_returns_0() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![1u8, 1, 1, 0];
        mgr.init(&part);
        assert_eq!(mgr.select_togo(), 0);
    }

    #[test]
    fn test_select_togo_known_returns_1() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![1u8, 1, 0, 0];
        mgr.init(&part);
        assert_eq!(mgr.select_togo(), 1);
    }

    #[test]
    fn test_constr_mgr_interface_init() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        ConstrMgrInterface::init(&mut mgr, &part);
        assert_eq!(mgr.diff[0], 2);
        assert_eq!(mgr.diff[1], 2);
    }

    #[test]
    fn test_constr_mgr_interface_check_legal() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        ConstrMgrInterface::init(&mut mgr, &part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        let check = ConstrMgrInterface::check_legal(&mut mgr, &move_info);
        assert_eq!(check, LegalCheck::GetBetter);
    }

    #[test]
    fn test_constr_mgr_interface_check_constraints() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        ConstrMgrInterface::init(&mut mgr, &part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert!(ConstrMgrInterface::check_constraints(&mgr, &move_info));
    }

    #[test]
    fn test_constr_mgr_interface_update_move() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        ConstrMgrInterface::init(&mut mgr, &part);
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        let _ = ConstrMgrInterface::check_legal(&mut mgr, &move_info);
        ConstrMgrInterface::update_move(&mut mgr, &move_info);
        assert_eq!(mgr.diff[0], 1);
        assert_eq!(mgr.diff[1], 3);
    }

    #[test]
    fn test_constr_mgr_interface_select_togo() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 0, 0];
        ConstrMgrInterface::init(&mut mgr, &part);
        let togo = ConstrMgrInterface::select_togo(&mgr);
        assert!(togo == 0 || togo == 1);
    }

    #[test]
    fn test_constr_mgr_interface_final_check() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.5);
        let part = vec![0u8, 0, 1, 1];
        assert!(ConstrMgrInterface::final_check(&mut mgr, &part));
    }

    // ── Ported from Python test_FMConstrMgr.py ─────────────────────

    #[test]
    fn test_chain_move_legal_checks() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.25);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);
        // diff = [2, 2], lowerbound = 1

        // Move vertex 0 from part 0 to part 1 → AllSatisfied
        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert_eq!(mgr.check_legal(&move_info), LegalCheck::AllSatisfied);

        mgr.update_move(&move_info);
        // diff = [1, 3]

        // Move vertex 1 from part 0 to part 1 → NotSatisfied
        let move_info2 = MoveInfoV {
            v: NodeIndex::new(1),
            from_part: 0,
            to_part: 1,
        };
        assert_eq!(mgr.check_legal(&move_info2), LegalCheck::NotSatisfied);

        // Move vertex 2 from part 1 to part 0 → AllSatisfied
        let move_info3 = MoveInfoV {
            v: NodeIndex::new(2),
            from_part: 1,
            to_part: 0,
        };
        assert_eq!(mgr.check_legal(&move_info3), LegalCheck::AllSatisfied);
    }

    #[test]
    fn test_chain_move_constraints() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.25);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);

        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert!(mgr.check_constraints(&move_info));

        // Rust update_move requires weight_cache from check_legal
        let _ = mgr.check_legal(&move_info);
        mgr.update_move(&move_info);

        let move_info2 = MoveInfoV {
            v: NodeIndex::new(1),
            from_part: 0,
            to_part: 1,
        };
        assert!(!mgr.check_constraints(&move_info2));
    }

    #[test]
    fn test_update_move_changes_diff() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut mgr = FMConstrMgr::new(netlist, 0.25);
        let part = vec![0u8, 0, 1, 1];
        mgr.init(&part);
        assert_eq!(mgr.diff, vec![2, 2]);

        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        let _ = mgr.check_legal(&move_info);
        mgr.update_move(&move_info);
        assert_eq!(mgr.diff, vec![1, 3]);
    }

    #[test]
    fn test_legal_3_parts_all_satisfied() {
        let netlist = SimpleNetlist::new(6, 0);
        let mut mgr = FMConstrMgr::with_num_parts(netlist, 0.3, 3);
        let part = vec![0u8, 0, 1, 1, 2, 2];
        mgr.init(&part);
        // diff = [2, 2, 2], totalweight = 6, totalweightK = 6 * 2/3 = 4, lowerbound = round(4*0.3) = 1

        let move_info = MoveInfoV {
            v: NodeIndex::new(0),
            from_part: 0,
            to_part: 1,
        };
        assert_eq!(mgr.check_legal(&move_info), LegalCheck::AllSatisfied);
    }
}
