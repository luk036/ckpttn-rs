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
}
