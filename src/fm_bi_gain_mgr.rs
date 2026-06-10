use crate::fm_bi_gain_calc::FMBiGainCalc;
use crate::fm_gain_mgr::{FMGainMgr, GainCalcTrait};
use crate::hypergraph::Hypergraph;

impl<Gnl: Hypergraph> GainCalcTrait<Gnl> for FMBiGainCalc<Gnl> {
    fn init(&mut self, part: &[u8]) -> i32 {
        self.init(part)
    }

    fn update_move_init(&mut self) {
        self.update_move_init()
    }

    fn init_idx_vec(&mut self, v: Gnl::Node, net: Gnl::Node) {
        self.init_idx_vec(v, net)
    }

    fn idx_vec(&self) -> &Vec<Gnl::Node> {
        &self.idx_vec
    }

    fn update_move_2pin_net(
        &mut self,
        part: &[u8],
        move_info: &crate::moveinfo::MoveInfo<Gnl::Node>,
    ) -> Gnl::Node {
        self.update_move_2pin_net(part, move_info)
    }

    fn update_move_3pin_net(
        &mut self,
        part: &[u8],
        move_info: &crate::moveinfo::MoveInfo<Gnl::Node>,
    ) -> Vec<i32> {
        self.update_move_3pin_net(part, move_info)
    }

    fn update_move_general_net(
        &mut self,
        part: &[u8],
        move_info: &crate::moveinfo::MoveInfo<Gnl::Node>,
    ) -> Vec<i32> {
        self.update_move_general_net(part, move_info)
    }

    fn delta_gain_w(&self) -> i32 {
        self.delta_gain_w()
    }
}

/// Binary Fiduccia-Mattheyses Gain Manager
///
/// Specialized for 2-way partitioning.
/// Ported from C++ `FMBiGainMgr` in `FMBiGainMgr.hpp`.
pub type FMBiGainMgr<Gnl> = FMGainMgr<Gnl, FMBiGainCalc<Gnl>>;

#[cfg(test)]
mod tests {
    use super::FMBiGainMgr;
    use crate::hypergraph::SimpleNetlist;
    use crate::fm_bi_gain_calc::FMBiGainCalc;
    use crate::fm_gain_mgr::GainCalcTrait;
    use crate::moveinfo::MoveInfo;
    use petgraph::graph::NodeIndex;

    fn make_nl() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        netlist
    }

    #[test]
    fn test_gain_calc_trait_init() {
        let mut calc = FMBiGainCalc::new(make_nl(), 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = GainCalcTrait::init(&mut calc, &part);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_gain_calc_trait_update_move_init() {
        let mut calc = FMBiGainCalc::new(make_nl(), 2);
        GainCalcTrait::update_move_init(&mut calc);
    }

    #[test]
    fn test_gain_calc_trait_init_idx_vec() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let mut calc = FMBiGainCalc::new(make_nl(), 2);
        GainCalcTrait::init_idx_vec(&mut calc, nodes[4], nodes[4]);
        let idx = GainCalcTrait::idx_vec(&calc);
        assert!(idx.contains(&nodes[0]) || idx.contains(&nodes[1]));
    }

    #[test]
    fn test_gain_calc_trait_2pin() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let mut calc = FMBiGainCalc::new(make_nl(), 2);
        let part = vec![0u8, 0, 1, 1];
        let move_info = MoveInfo {
            net: nodes[4],
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        let w = GainCalcTrait::update_move_2pin_net(&mut calc, &part, &move_info);
        assert_eq!(w, nodes[1]);
    }

    #[test]
    fn test_gain_calc_trait_delta_gain_w() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let mut calc = FMBiGainCalc::new(make_nl(), 2);
        let part = vec![0u8, 0, 1, 1];
        let move_info = MoveInfo {
            net: nodes[4],
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        let _ = GainCalcTrait::update_move_2pin_net(&mut calc, &part, &move_info);
        let dg = GainCalcTrait::delta_gain_w(&calc);
        assert_eq!(dg, 2);
    }

    #[test]
    fn test_fm_bi_gain_mgr_creation() {
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let _mgr = FMBiGainMgr::new(make_nl(), calc, 2);
    }
}
