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
