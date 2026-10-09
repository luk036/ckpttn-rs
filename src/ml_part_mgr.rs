use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::fm_constr_mgr::LegalCheck;
use crate::hypergraph::Hypergraph;
use crate::min_cover::contract_subgraph;

/// Per-family hooks for the shared multi-level skeleton.
///
/// Each implementation selects the (gain, constraint, partition-manager) triple
/// for one algorithm family ({FM, NN} x {bi, k-way}); the recursive skeleton is
/// written once in [`run_partition_with`].
pub(crate) trait MlPartMgrSpec {
    fn legalize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> (LegalCheck, i32);

    fn optimize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> i32;
}

pub(crate) struct BiFmSpec;
pub(crate) struct KWayFmSpec;
pub(crate) struct BiNnSpec;
pub(crate) struct KWayNnSpec;

impl MlPartMgrSpec for BiFmSpec {
    fn legalize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        _num_parts: u8,
        part: &mut [u8],
    ) -> (LegalCheck, i32) {
        use crate::fm_bi_constr_mgr::FMBiConstrMgr;
        use crate::fm_bi_gain_calc::FMBiGainCalc;
        use crate::fm_bi_gain_mgr::FMBiGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, bal_tol);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, 2);
        let legalcheck = part_mgr.legalize(part);
        (legalcheck, part_mgr.total_cost)
    }

    fn optimize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        _num_parts: u8,
        part: &mut [u8],
    ) -> i32 {
        use crate::fm_bi_constr_mgr::FMBiConstrMgr;
        use crate::fm_bi_gain_calc::FMBiGainCalc;
        use crate::fm_bi_gain_mgr::FMBiGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, bal_tol);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, 2);
        part_mgr.optimize(part);
        part_mgr.total_cost
    }
}

impl MlPartMgrSpec for KWayFmSpec {
    fn legalize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> (LegalCheck, i32) {
        use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        use crate::fm_kway_gain_mgr::FMKWayGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMKWayGainCalc::new(hyprgraph, num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, bal_tol, num_parts);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, num_parts as usize);
        let legalcheck = part_mgr.legalize(part);
        (legalcheck, part_mgr.total_cost)
    }

    fn optimize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> i32 {
        use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        use crate::fm_kway_gain_mgr::FMKWayGainMgr;
        use crate::part_mgr_base::PartMgrBase;

        let gain_calc = FMKWayGainCalc::new(hyprgraph, num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, bal_tol, num_parts);
        let mut part_mgr = PartMgrBase::new(hyprgraph, gain_mgr, constr_mgr, num_parts as usize);
        part_mgr.optimize(part);
        part_mgr.total_cost
    }
}

impl MlPartMgrSpec for BiNnSpec {
    fn legalize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        _num_parts: u8,
        part: &mut [u8],
    ) -> (LegalCheck, i32) {
        use crate::fm_bi_constr_mgr::FMBiConstrMgr;
        use crate::fm_bi_gain_calc::FMBiGainCalc;
        use crate::fm_bi_gain_mgr::FMBiGainMgr;
        use crate::nn_part_mgr::NNPartMgr;

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, bal_tol);
        let mut part_mgr = NNPartMgr::new(hyprgraph, gain_mgr, constr_mgr, 2);
        let legalcheck = part_mgr.legalize(part);
        (legalcheck, part_mgr.total_cost)
    }

    fn optimize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        _num_parts: u8,
        part: &mut [u8],
    ) -> i32 {
        use crate::fm_bi_constr_mgr::FMBiConstrMgr;
        use crate::fm_bi_gain_calc::FMBiGainCalc;
        use crate::fm_bi_gain_mgr::FMBiGainMgr;
        use crate::nn_part_mgr::NNPartMgr;

        let gain_calc = FMBiGainCalc::new(hyprgraph, 2);
        let gain_mgr = FMBiGainMgr::new(hyprgraph, gain_calc, 2);
        let constr_mgr = FMBiConstrMgr::new(hyprgraph, bal_tol);
        let mut part_mgr = NNPartMgr::new(hyprgraph, gain_mgr, constr_mgr, 2);
        part_mgr.optimize(part);
        part_mgr.total_cost
    }
}

impl MlPartMgrSpec for KWayNnSpec {
    fn legalize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> (LegalCheck, i32) {
        use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        use crate::fm_kway_gain_mgr::FMKWayGainMgr;
        use crate::nn_part_mgr::NNPartMgr;

        let gain_calc = FMKWayGainCalc::new(hyprgraph, num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, bal_tol, num_parts);
        let mut part_mgr = NNPartMgr::new(hyprgraph, gain_mgr, constr_mgr, num_parts as usize);
        let legalcheck = part_mgr.legalize(part);
        (legalcheck, part_mgr.total_cost)
    }

    fn optimize<G: Hypergraph<Node = NodeIndex>>(
        hyprgraph: &G,
        bal_tol: f64,
        num_parts: u8,
        part: &mut [u8],
    ) -> i32 {
        use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        use crate::fm_kway_gain_mgr::FMKWayGainMgr;
        use crate::nn_part_mgr::NNPartMgr;

        let gain_calc = FMKWayGainCalc::new(hyprgraph, num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, bal_tol, num_parts);
        let mut part_mgr = NNPartMgr::new(hyprgraph, gain_mgr, constr_mgr, num_parts as usize);
        part_mgr.optimize(part);
        part_mgr.total_cost
    }
}

/// Shared multi-level recursion: legalize, optionally contract + recurse, then
/// optimize at this level. `S` supplies the algorithm family.
pub(crate) fn run_partition_with<S: MlPartMgrSpec, G: Hypergraph<Node = NodeIndex>>(
    hyprgraph: &G,
    module_weight: &[u32],
    part: &mut [u8],
    bal_tol: f64,
    num_parts: u8,
    limitsize: usize,
) -> (LegalCheck, i32) {
    let (legalcheck, total_cost) = S::legalize(hyprgraph, bal_tol, num_parts, part);
    if legalcheck != LegalCheck::AllSatisfied {
        return (legalcheck, total_cost);
    }

    if hyprgraph.number_of_modules() >= limitsize {
        let (hgr2, module_weight2) = contract_subgraph(hyprgraph, module_weight, &HashSet::new());
        if hgr2.number_of_modules() * 3 / 2 < hyprgraph.number_of_modules() {
            let mut part2 = vec![0u8; hgr2.number_of_modules()];
            hgr2.projection_up(part, &mut part2);
            let (lc_recur, _) = run_partition_with::<S, _>(
                &hgr2,
                &module_weight2,
                &mut part2,
                bal_tol,
                num_parts,
                limitsize,
            );
            if lc_recur == LegalCheck::AllSatisfied {
                hgr2.projection_down(&part2, part);
            }
        }
    }

    (legalcheck, S::optimize(hyprgraph, bal_tol, num_parts, part))
}

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
    #[inline]
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
        let (legalcheck, total_cost) = run_partition_with::<BiFmSpec, _>(
            hyprgraph,
            module_weight,
            part,
            self.bal_tol,
            2,
            self.limitsize,
        );
        self.total_cost = total_cost;
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
        let (legalcheck, total_cost) = run_partition_with::<KWayFmSpec, _>(
            hyprgraph,
            module_weight,
            part,
            self.bal_tol,
            self.num_parts,
            self.limitsize,
        );
        self.total_cost = total_cost;
        legalcheck
    }
}

/// Bi-partition multi-level manager using the NNPartMgr pure local search.
pub struct MLBiNNPartMgr {
    pub bal_tol: f64,
    pub total_cost: i32,
    pub limitsize: usize,
}

impl MLBiNNPartMgr {
    pub fn new(bal_tol: f64) -> Self {
        MLBiNNPartMgr {
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
        let (legalcheck, total_cost) = run_partition_with::<BiNnSpec, _>(
            hyprgraph,
            module_weight,
            part,
            self.bal_tol,
            2,
            self.limitsize,
        );
        self.total_cost = total_cost;
        legalcheck
    }
}

/// K-way multi-level manager using the NNPartMgr pure local search.
pub struct MLKWayNNPartMgr {
    pub bal_tol: f64,
    pub num_parts: u8,
    pub total_cost: i32,
    pub limitsize: usize,
}

impl MLKWayNNPartMgr {
    pub fn new(bal_tol: f64, num_parts: u8) -> Self {
        MLKWayNNPartMgr {
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
        let (legalcheck, total_cost) = run_partition_with::<KWayNnSpec, _>(
            hyprgraph,
            module_weight,
            part,
            self.bal_tol,
            self.num_parts,
            self.limitsize,
        );
        self.total_cost = total_cost;
        legalcheck
    }
}

#[cfg(test)]
mod tests {
    use petgraph::graph::NodeIndex;

    use crate::hypergraph::{Hypergraph, SimpleNetlist};

    fn create_dwarf_netlist() -> (SimpleNetlist, Vec<u32>) {
        let netlist = crate::test_support::create_dwarf_netlist();
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
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        assert_eq!(result, super::LegalCheck::AllSatisfied);
        assert!(
            mgr.total_cost > 0,
            "dwarf bi ML collapsed to a degenerate cost-0 partition"
        );
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

    #[test]
    fn test_ml_bi_nn_part_mgr_dwarf() {
        let (netlist, weights) = create_dwarf_netlist();
        let mut mgr = super::MLBiNNPartMgr::new(0.3);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    fn test_ml_bi_nn_part_mgr_test_netlist() {
        let (netlist, weights) = create_test_netlist();
        let mut mgr = super::MLBiNNPartMgr::new(0.4);
        let mut part = vec![0u8; netlist.number_of_modules()];
        let result = mgr.run_partition(&netlist, &weights, &mut part);
        assert!(
            result == super::LegalCheck::AllSatisfied || result == super::LegalCheck::NotSatisfied
        );
    }

    #[test]
    fn test_ml_kway_nn_part_mgr_dwarf() {
        let (netlist, weights) = create_dwarf_netlist();
        let mut mgr = super::MLKWayNNPartMgr::new(0.4, 3);
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
}
