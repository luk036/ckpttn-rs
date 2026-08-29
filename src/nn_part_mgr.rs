use crate::fm_constr_mgr::LegalCheck;
use crate::fm_gain_mgr::{ConstrMgrInterface, GainMgrInterface};
use crate::hypergraph::Hypergraph;
use crate::part_mgr_base::PartMgr;

/// No-Nonsense Partition Manager.
///
/// Implements a simplified FM-style partition optimization without
/// backtracking (no snapshot/restore). Unlike `PartMgrBase`, `optimize_1pass`
/// only accepts positive-gain moves and stops at the first non-improving move.
/// The shared skeleton (`init`/`legalize`/`optimize`/`final_check`) is
/// inherited from the private `PartMgr` trait (Template Method).
/// Ported from Python `NNPartMgr` in `NNPartMgr.py`.
pub struct NNPartMgr<Gnl: Hypergraph, GainMgr, ConstrMgr> {
    pub hyprgraph: Gnl,
    pub gain_mgr: GainMgr,
    pub validator: ConstrMgr,
    pub num_parts: usize,
    pub total_cost: i32,
}

impl<Gnl, GainMgr, ConstrMgr> NNPartMgr<Gnl, GainMgr, ConstrMgr>
where
    Gnl: Hypergraph,
    GainMgr: GainMgrInterface<Gnl>,
    ConstrMgr: ConstrMgrInterface<Gnl>,
{
    pub fn new(hyprgraph: Gnl, gain_mgr: GainMgr, validator: ConstrMgr, num_parts: usize) -> Self {
        NNPartMgr {
            hyprgraph,
            gain_mgr,
            validator,
            num_parts,
            total_cost: 0,
        }
    }

    #[inline]
    pub fn init(&mut self, part: &mut [u8]) {
        PartMgr::init(self, part)
    }

    #[inline]
    pub fn legalize(&mut self, part: &mut [u8]) -> LegalCheck {
        PartMgr::legalize(self, part)
    }

    #[inline]
    pub fn optimize(&mut self, part: &mut [u8]) {
        PartMgr::optimize(self, part)
    }

    #[inline]
    pub fn final_check(&mut self, part: &mut [u8]) -> bool {
        PartMgr::final_check(self, part)
    }
}

impl<Gnl, GainMgr, ConstrMgr> PartMgr<Gnl, GainMgr, ConstrMgr>
    for NNPartMgr<Gnl, GainMgr, ConstrMgr>
where
    Gnl: Hypergraph,
    GainMgr: GainMgrInterface<Gnl>,
    ConstrMgr: ConstrMgrInterface<Gnl>,
{
    #[inline]
    fn hyprgraph(&self) -> &Gnl {
        &self.hyprgraph
    }

    #[inline]
    fn gain_mgr(&mut self) -> &mut GainMgr {
        &mut self.gain_mgr
    }

    #[inline]
    fn validator(&mut self) -> &mut ConstrMgr {
        &mut self.validator
    }

    #[inline]
    fn total_cost(&self) -> i32 {
        self.total_cost
    }

    #[inline]
    fn total_cost_mut(&mut self) -> &mut i32 {
        &mut self.total_cost
    }

    #[inline]
    fn max_passes(&self) -> usize {
        usize::MAX
    }

    fn optimize_1pass(&mut self, part: &mut [u8]) {
        let mut totalgain = 0i32;

        while !self.gain_mgr().is_empty() {
            let (move_info_v, gainmax) = self.gain_mgr().select(part);
            if gainmax <= 0 {
                break;
            }
            let satisfied_ok = self.validator().check_constraints(&move_info_v);
            if !satisfied_ok {
                continue;
            }
            self.gain_mgr().update_move(part, &move_info_v);
            self.gain_mgr().update_move_v(&move_info_v, gainmax);
            self.validator().update_move(&move_info_v);
            totalgain += gainmax;
            part[self.hyprgraph().module_index(move_info_v.v)] = move_info_v.to_part;
        }
        *self.total_cost_mut() -= totalgain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fm_bi_gain_calc::FMBiGainCalc;
    use crate::fm_constr_mgr::FMConstrMgr;
    use crate::hypergraph::SimpleNetlist;
    use crate::nn_gain_mgr::NNGainMgr;
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
    fn test_nn_part_mgr_new() {
        let hyprgraph = make_nl();
        let gain_calc = FMBiGainCalc::new(make_nl(), 2);
        let gain_mgr = NNGainMgr::new(make_nl(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl(), 0.5);
        let pm = NNPartMgr::new(hyprgraph, gain_mgr, validator, 2);
        assert_eq!(pm.num_parts, 2);
        assert_eq!(pm.total_cost, 0);
    }

    #[test]
    fn test_nn_part_mgr_init() {
        let hyprgraph = make_nl();
        let gain_calc = FMBiGainCalc::new(make_nl(), 2);
        let gain_mgr = NNGainMgr::new(make_nl(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl(), 0.5);
        let mut pm = NNPartMgr::new(hyprgraph, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 1, 1];
        pm.init(&mut part);
        assert_eq!(pm.total_cost, 0);
    }

    #[test]
    fn test_nn_part_mgr_final_check() {
        let hyprgraph = make_nl();
        let gain_calc = FMBiGainCalc::new(make_nl(), 2);
        let gain_mgr = NNGainMgr::new(make_nl(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl(), 0.5);
        let mut pm = NNPartMgr::new(hyprgraph, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 1, 1];
        pm.init(&mut part);
        let legal = pm.final_check(&mut part);
        assert!(legal);
    }
}
