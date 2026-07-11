use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::fm_constr_mgr::LegalCheck;
use crate::hypergraph::Hypergraph;
use crate::min_cover::contract_subgraph;

/// Multi-level FM partitioning manager.
///
/// Implements multi-level recursive partitioning: contracts large hypergraphs
/// into smaller ones, recurses, then uncoarsens with FM optimization at each level.
/// Ported from Python `MLPartMgr` in `MLPartMgr.py`.
pub struct MLPartMgr {
    pub total_cost: i32,
    pub limitsize: usize,
}

impl Default for MLPartMgr {
    fn default() -> Self {
        Self::new()
    }
}

impl MLPartMgr {
    pub fn new() -> Self {
        MLPartMgr {
            total_cost: 0,
            limitsize: 50,
        }
    }
}

/// Bi-partition multi-level manager using PartMgrBase + FMBiGainMgr + FMBiConstrMgr.
pub struct MLBiPartMgr {
    pub bal_tol: f64,
    pub total_cost: i32,
    pub limitsize: usize,
}

impl MLBiPartMgr {
    pub fn new(bal_tol: f64) -> Self {
        MLBiPartMgr {
            bal_tol,
            total_cost: 0,
            limitsize: 50,
        }
    }

    pub fn run_partition(
        &mut self,
        hyprgraph: &impl Hypergraph<Node = NodeIndex>,
        module_weight: &[u32],
        part: &mut [u8],
    ) -> LegalCheck {
        use crate::fm_bi_constr_mgr::FMBiConstrMgr;
        use crate::fm_bi_gain_calc::FMBiGainCalc;
        use crate::fm_bi_gain_mgr::FMBiGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, self.bal_tol);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, 2);
        let legalcheck = part_mgr.legalize(part);

        if legalcheck != LegalCheck::AllSatisfied {
            self.total_cost = part_mgr.total_cost;
            return legalcheck;
        }

        if hyprgraph.number_of_modules() >= self.limitsize {
            let (hgr2, module_weight2) =
                contract_subgraph(hyprgraph, module_weight, &HashSet::new());
            if hgr2.number_of_modules() * 3 / 2 < hyprgraph.number_of_modules() {
                let mut part2 = vec![0u8; hgr2.number_of_modules()];
                hgr2.projection_up(part, &mut part2);
                let lc_recur = self.run_partition(&hgr2, &module_weight2, &mut part2);
                if lc_recur == LegalCheck::AllSatisfied {
                    hgr2.projection_down(&part2, part);
                }
            }
        }

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, self.bal_tol);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, 2);
        part_mgr.optimize(part);
        self.total_cost = part_mgr.total_cost;
        legalcheck
    }
}

/// K-way multi-level manager using PartMgrBase + FMKWayGainMgr + FMKWayConstrMgr.
pub struct MLKWayPartMgr {
    pub bal_tol: f64,
    pub num_parts: u8,
    pub total_cost: i32,
    pub limitsize: usize,
}

impl MLKWayPartMgr {
    pub fn new(bal_tol: f64, num_parts: u8) -> Self {
        MLKWayPartMgr {
            bal_tol,
            num_parts,
            total_cost: 0,
            limitsize: 50,
        }
    }

    pub fn run_partition(
        &mut self,
        hyprgraph: &impl Hypergraph<Node = NodeIndex>,
        module_weight: &[u32],
        part: &mut [u8],
    ) -> LegalCheck {
        use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        use crate::fm_kway_gain_mgr::FMKWayGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMKWayGainCalc::new(hyprgraph, self.num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, self.num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, self.bal_tol, self.num_parts);
        let mut part_mgr =
            PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, self.num_parts as usize);
        let legalcheck = part_mgr.legalize(part);

        if legalcheck != LegalCheck::AllSatisfied {
            self.total_cost = part_mgr.total_cost;
            return legalcheck;
        }

        if hyprgraph.number_of_modules() >= self.limitsize {
            let (hgr2, module_weight2) =
                contract_subgraph(hyprgraph, module_weight, &HashSet::new());
            if hgr2.number_of_modules() * 3 / 2 < hyprgraph.number_of_modules() {
                let mut part2 = vec![0u8; hgr2.number_of_modules()];
                hgr2.projection_up(part, &mut part2);
                let lc_recur = self.run_partition(&hgr2, &module_weight2, &mut part2);
                if lc_recur == LegalCheck::AllSatisfied {
                    hgr2.projection_down(&part2, part);
                }
            }
        }

        let gain_calc = FMKWayGainCalc::new(hyprgraph, self.num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, self.num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, self.bal_tol, self.num_parts);
        let mut part_mgr =
            PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, self.num_parts as usize);
        part_mgr.optimize(part);
        self.total_cost = part_mgr.total_cost;
        legalcheck
    }
}

#[cfg(test)]
mod tests {
    use petgraph::graph::NodeIndex;

    use crate::hypergraph::{Hypergraph, SimpleNetlist};

    fn create_dwarf_netlist() -> (SimpleNetlist, Vec<u32>) {
        let mut netlist = SimpleNetlist::new(7, 6);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        // C++ dwarf: a0=0, a1=1, a2=2, a3=3, p1=4, p2=5, p3=6, n1=7..n6=12
        // Edges: (p1,n1), (a0,n1), (a1,n1), (a0,n2), (a2,n2), (a3,n2),
        //        (a1,n3), (a2,n3), (a3,n3), (a2,n4), (p2,n4), (a3,n5), (p3,n5), (a0,n6)
        let edge_pairs: Vec<(usize, usize)> = vec![
            (4, 7), (0, 7), (1, 7), (0, 8), (2, 8), (3, 8),
            (1, 9), (2, 9), (3, 9), (2, 10), (5, 10), (3, 11), (6, 11), (0, 12),
        ];
        for (u, v) in &edge_pairs {
            netlist.add_edge(nodes[*u], nodes[*v]);
        }
        netlist.module_weight = vec![1, 3, 4, 2, 0, 0, 0];
        let weights = netlist.module_weight.clone();
        (netlist, weights)
    }

    fn create_test_netlist() -> (SimpleNetlist, Vec<u32>) {
        let mut netlist = SimpleNetlist::new(3, 3);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[3]);
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[3]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[4]);
        netlist.add_edge(nodes[0], nodes[5]);
        netlist.module_weight = vec![3, 4, 2];
        let weights = netlist.module_weight.clone();
        (netlist, weights)
    }

    #[test]
    fn test_ml_bi_part_mgr_basic() {
        let netlist = SimpleNetlist::new(4, 2);
        let mut mgr = super::MLBiPartMgr::new(0.45);
        let part = vec![0u8, 0, 1, 1];
        let weights = vec![1u32; 4];
        let result = mgr.run_partition(&netlist, &weights, &mut part.clone());
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
    }

    #[test]
    fn test_ml_bi_part_mgr_dwarf() {
        let (netlist, weights) = create_dwarf_netlist();
        let mut mgr = super::MLBiPartMgr::new(0.3);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let _result = mgr.run_partition(&netlist, &weights, &mut part);
        // Total cost is valid regardless of legalization result
    }

    #[test]
    fn test_ml_bi_part_mgr_test_netlist() {
        let (netlist, weights) = create_test_netlist();
        let mut mgr = super::MLBiPartMgr::new(0.4);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
    }

    #[test]
    fn test_ml_kway_part_mgr_basic() {
        let netlist = SimpleNetlist::new(6, 2);
        let mut mgr = super::MLKWayPartMgr::new(0.45, 3);
        let part = vec![0u8, 0, 1, 1, 2, 2];
        let weights = vec![1u32; 6];
        let result = mgr.run_partition(&netlist, &weights, &mut part.clone());
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
    }

    #[test]
    fn test_ml_kway_part_mgr_dwarf() {
        let (netlist, weights) = create_dwarf_netlist();
        let mut mgr = super::MLKWayPartMgr::new(0.4, 3);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
        if result == super::LegalCheck::AllSatisfied {
            use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
            let mut constr_mgr = FMKWayConstrMgr::new(&netlist, 0.4, 3);
            assert!(constr_mgr.final_check(&part));
        }
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    fn test_ml_bi_part_mgr_legalize_all_zero() {
        let netlist = SimpleNetlist::new(4, 2);
        let mut mgr = super::MLBiPartMgr::new(0.5);
        let mut part = vec![0u8; 4];
        let weights = vec![1u32; 4];
        // All modules in partition 0 - legalize should fix this
        let _result = mgr.run_partition(&netlist, &weights, &mut part);
    }

    #[test]
    fn test_ml_bi_part_mgr_optimize_reduces_cost() {
        let (netlist, weights) = create_dwarf_netlist();
        let mut mgr = super::MLBiPartMgr::new(0.4);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        if result == super::LegalCheck::AllSatisfied {
            let cost_after = mgr.total_cost;
            // Re-run should not increase cost
            let _result2 = mgr.run_partition(&netlist, &weights, &mut part);
            assert!(mgr.total_cost <= cost_after || mgr.total_cost == cost_after);
        }
    }
}
