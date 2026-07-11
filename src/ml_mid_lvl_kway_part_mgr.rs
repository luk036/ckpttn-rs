use std::collections::HashSet;

use petgraph::graph::NodeIndex;

use crate::fm_constr_mgr::LegalCheck;
use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
use crate::fm_kway_gain_calc::FMKWayGainCalc;
use crate::fm_kway_gain_mgr::FMKWayGainMgr;
use crate::fm_part_mgr::FMPartMgr;
use crate::hypergraph::Hypergraph;
use crate::mid_lvl_kway_part_mgr::MidLvlKWayPartMgr;
use crate::min_cover::contract_subgraph;

pub struct MLMidLvlKWayPartMgr {
    bal_tol: f64,
    num_parts: u8,
    limitsize: usize,
    pub total_cost: i32,
}

const BASE_EXHAUSTIVE: usize = 25;

impl MLMidLvlKWayPartMgr {
    pub fn new(bal_tol: f64, num_parts: u8) -> Self {
        MLMidLvlKWayPartMgr {
            bal_tol,
            num_parts,
            limitsize: 50,
            total_cost: 0,
        }
    }

    pub fn set_limitsize(&mut self, limit: usize) {
        self.limitsize = limit;
    }

    pub fn optimize(&mut self, part: &mut [u8], hyprgraph: &impl Hypergraph<Node = NodeIndex>) {
        let exhaustive_limit = (BASE_EXHAUSTIVE * self.num_parts as usize) / 2;

        if hyprgraph.number_of_modules() <= exhaustive_limit {
            let mut kway_mgr = MidLvlKWayPartMgr::new(self.bal_tol, self.num_parts);
            kway_mgr.optimize(part, hyprgraph);
            self.total_cost = kway_mgr.total_cost;
            return;
        }

        let gain_calc = FMKWayGainCalc::new(hyprgraph, self.num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, self.num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, self.bal_tol, self.num_parts);
        let mut part_mgr = FMPartMgr::new(hyprgraph, gain_mgr, constr_mgr, self.num_parts as usize);
        let lc = part_mgr.legalize(part);
        if lc != LegalCheck::AllSatisfied {
            return;
        }

        if hyprgraph.number_of_modules() >= self.limitsize {
            let module_weight: Vec<u32> = hyprgraph
                .modules()
                .map(|v| hyprgraph.get_module_weight(v))
                .collect();
            let (hgr2, _module_weight2) =
                contract_subgraph(hyprgraph, &module_weight, &HashSet::new());
            if hgr2.number_of_modules() * 3 / 2 < hyprgraph.number_of_modules() {
                let mut part2 = vec![0u8; hgr2.number_of_modules()];
                hgr2.projection_up(part, &mut part2);
                self.optimize(&mut part2, &hgr2);
                hgr2.projection_down(&part2, part);
            }
        }

        let gain_calc = FMKWayGainCalc::new(hyprgraph, self.num_parts);
        let gain_mgr = FMKWayGainMgr::new(hyprgraph, gain_calc, self.num_parts);
        let constr_mgr = FMKWayConstrMgr::new(hyprgraph, self.bal_tol, self.num_parts);
        let mut part_mgr = FMPartMgr::new(hyprgraph, gain_mgr, constr_mgr, self.num_parts as usize);
        part_mgr.optimize(part);
        self.total_cost = part_mgr.total_cost;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    fn create_dwarf_netlist() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(7, 6);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        let edges: Vec<(usize, usize)> = vec![
            (4, 7),
            (0, 7),
            (1, 7),
            (0, 8),
            (2, 8),
            (3, 8),
            (1, 9),
            (2, 9),
            (3, 9),
            (2, 10),
            (5, 10),
            (3, 11),
            (6, 11),
            (0, 12),
        ];
        for (u, v) in &edges {
            netlist.add_edge(nodes[*u], nodes[*v]);
        }
        netlist.module_weight = vec![1, 3, 4, 2, 0, 0, 0];
        netlist
    }

    #[test]
    #[ignore = "MLMidLvlKWay: depends on MidLvlKWayPartMgr which hangs"]
    fn test_ml_mid_lvl_kway_dwarf_3way() {
        let hyprgraph = create_dwarf_netlist();
        let n = hyprgraph.number_of_modules();
        let mut mgr = MLMidLvlKWayPartMgr::new(0.4, 3);
        let mut part = vec![0u8; n];
        for i in 0..n {
            part[i] = (i % 3) as u8;
        }
        mgr.optimize(&mut part, &hyprgraph);
        assert!(mgr.total_cost >= 0);
    }
}
