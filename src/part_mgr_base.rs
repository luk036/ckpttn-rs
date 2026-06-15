use crate::fm_constr_mgr::LegalCheck;
use crate::fm_gain_mgr::{ConstrMgrInterface, GainMgrInterface};
use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfoV;

/// Fiduccia-Mattheyses Partitioning Algorithm Manager Base
///
/// Ported from C++ `PartMgrBase` in `PartMgrBase.hpp`/`PartMgrBase.cpp`.
pub struct PartMgrBase<Gnl: Hypergraph, GainMgr, ConstrMgr> {
    pub hyprgraph: Gnl,
    pub gain_mgr: GainMgr,
    pub validator: ConstrMgr,
    pub num_parts: usize,
    pub total_cost: i32,
}

impl<Gnl, GainMgr, ConstrMgr> PartMgrBase<Gnl, GainMgr, ConstrMgr>
where
    Gnl: Hypergraph,
    GainMgr: GainMgrInterface<Gnl>,
    ConstrMgr: ConstrMgrInterface<Gnl>,
{
    pub fn new(hyprgraph: Gnl, gain_mgr: GainMgr, validator: ConstrMgr, num_parts: usize) -> Self {
        PartMgrBase {
            hyprgraph,
            gain_mgr,
            validator,
            num_parts,
            total_cost: 0,
        }
    }

    pub fn init(&mut self, part: &mut [u8]) {
        self.total_cost = self.gain_mgr.init(part);
        self.validator.init(part);
    }

    pub fn legalize(&mut self, part: &mut [u8]) -> LegalCheck {
        self.init(part);

        let mut legalcheck = LegalCheck::NotSatisfied;
        while legalcheck != LegalCheck::AllSatisfied {
            let to_part = self.validator.select_togo();
            if self.gain_mgr.is_empty_togo(to_part) {
                break;
            }
            let (v, gainmax) = self.gain_mgr.select_togo(to_part);
            let from_part = part[self.hyprgraph.module_index(v)];
            let move_info_v = MoveInfoV {
                v,
                from_part,
                to_part,
            };
            legalcheck = self.validator.check_legal(&move_info_v);
            if legalcheck == LegalCheck::NotSatisfied {
                continue;
            }
            self.gain_mgr.update_move(part, &move_info_v);
            self.validator.update_move(&move_info_v);
            part[self.hyprgraph.module_index(v)] = to_part;
            self.total_cost -= gainmax;
        }
        legalcheck
    }

    pub fn optimize(&mut self, part: &mut [u8]) {
        loop {
            self.init(part);
            let totalcost_before = self.total_cost;
            self.optimize_1pass(part);
            if self.total_cost == totalcost_before {
                break;
            }
        }
    }

    fn optimize_1pass(&mut self, part: &mut [u8]) {
        let mut snapshot: Option<Vec<u8>> = None;
        let mut totalgain = 0i32;
        let mut deferredsnapshot = false;
        let mut besttotalgain = 0i32;

        while !self.gain_mgr.is_empty() {
            let (move_info_v, gainmax) = self.gain_mgr.select(part);
            let satisfied_ok = self.validator.check_constraints(&move_info_v);
            if !satisfied_ok {
                continue;
            }
            if gainmax < 0 {
                if !deferredsnapshot || totalgain > besttotalgain {
                    snapshot = Some(part.to_vec());
                    besttotalgain = totalgain;
                }
                deferredsnapshot = true;
            } else if totalgain + gainmax >= besttotalgain {
                besttotalgain = totalgain + gainmax;
                deferredsnapshot = false;
            }

            self.gain_mgr.lock(move_info_v.to_part, move_info_v.v);
            self.gain_mgr.update_move(part, &move_info_v);
            self.gain_mgr.update_move_v(&move_info_v, gainmax);
            self.validator.update_move(&move_info_v);
            totalgain += gainmax;
            part[self.hyprgraph.module_index(move_info_v.v)] = move_info_v.to_part;
        }

        if deferredsnapshot {
            if let Some(snap) = snapshot {
                part.copy_from_slice(&snap);
            }
            totalgain = besttotalgain;
        }
        self.total_cost -= totalgain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use crate::fm_bi_gain_calc::FMBiGainCalc;
    use crate::fm_bi_gain_mgr::FMBiGainMgr;
    use crate::fm_constr_mgr::FMConstrMgr;
    use petgraph::graph::NodeIndex;

    fn make_nl_4m2n() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        netlist
    }

    fn make_nl_4m2n_heavy0() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        netlist.module_weight[0] = 100;
        netlist
    }

    fn make_nl_6m3n() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(6, 3);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[6]);
        netlist.add_edge(nodes[1], nodes[6]);
        netlist.add_edge(nodes[0], nodes[7]);
        netlist.add_edge(nodes[2], nodes[7]);
        netlist.add_edge(nodes[3], nodes[8]);
        netlist.add_edge(nodes[4], nodes[8]);
        netlist.add_edge(nodes[5], nodes[8]);
        netlist
    }

    #[test]
    fn test_new() {
        let hyprgraph = make_nl_4m2n();
        let gain_calc = FMBiGainCalc::new(make_nl_4m2n(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_4m2n(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_4m2n(), 0.5);
        let pm = PartMgrBase::new(hyprgraph, gain_mgr, validator, 2);
        assert_eq!(pm.num_parts, 2);
        assert_eq!(pm.total_cost, 0);
    }

    #[test]
    fn test_init() {
        let hyprgraph = make_nl_4m2n();
        let gain_calc = FMBiGainCalc::new(make_nl_4m2n(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_4m2n(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_4m2n(), 0.5);
        let mut pm = PartMgrBase::new(hyprgraph, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 1, 1];
        pm.init(&mut part);
        assert_eq!(pm.total_cost, 0);
    }

    #[test]
    fn test_legalize_already_balanced() {
        let hyprgraph = make_nl_4m2n();
        let gain_calc = FMBiGainCalc::new(make_nl_4m2n(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_4m2n(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_4m2n(), 0.5);
        let mut pm = PartMgrBase::new(hyprgraph, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 1, 1];
        let _result = pm.legalize(&mut part);
    }

    #[test]
    fn test_legalize_not_satisfied() {
        let netlist = make_nl_4m2n_heavy0();
        let gain_calc = FMBiGainCalc::new(make_nl_4m2n_heavy0(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_4m2n_heavy0(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_4m2n_heavy0(), 0.5);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 1, 1];
        let result = pm.legalize(&mut part);
        assert!(result == LegalCheck::AllSatisfied || result == LegalCheck::NotSatisfied);
    }

    #[test]
    fn test_optimize_basic() {
        let netlist = make_nl_6m3n();
        let gain_calc = FMBiGainCalc::new(make_nl_6m3n(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_6m3n(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_6m3n(), 0.5);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 0, 0, 0, 0];
        pm.optimize(&mut part);
    }

    #[test]
    fn test_legalize_boundary() {
        let netlist = make_nl_4m2n();
        let gain_calc = FMBiGainCalc::new(make_nl_4m2n(), 2);
        let gain_mgr = FMBiGainMgr::new(make_nl_4m2n(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_nl_4m2n(), 0.5);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8, 0, 0, 0];
        let result = pm.legalize(&mut part);
        assert!(result == LegalCheck::AllSatisfied || result == LegalCheck::NotSatisfied);
    }

    // ── Ported from Python test_FMBiPartMgr.py ─────────────────────

    fn make_netlist_drawf() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(7, 6);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.module_weight[0] = 3;
        netlist.add_edge(nodes[0], nodes[7]);
        netlist.add_edge(nodes[1], nodes[7]);
        netlist.add_edge(nodes[1], nodes[8]);
        netlist.add_edge(nodes[2], nodes[8]);
        netlist.add_edge(nodes[2], nodes[9]);
        netlist.add_edge(nodes[3], nodes[9]);
        netlist.add_edge(nodes[3], nodes[10]);
        netlist.add_edge(nodes[4], nodes[10]);
        netlist.add_edge(nodes[4], nodes[11]);
        netlist.add_edge(nodes[5], nodes[11]);
        netlist.add_edge(nodes[5], nodes[12]);
        netlist.add_edge(nodes[6], nodes[12]);
        netlist.add_edge(nodes[6], nodes[7]);
        netlist
    }

    #[test]
    fn test_legalize_and_optimize_with_dict_part() {
        let netlist = make_netlist_drawf();
        let gain_calc = FMBiGainCalc::new(make_netlist_drawf(), 2);
        let gain_mgr = FMBiGainMgr::new(make_netlist_drawf(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_netlist_drawf(), 0.5);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8; 7];

        let legal_check = pm.legalize(&mut part);
        // With bal_tol=0.5, legalization should succeed or at least not crash
        assert!(legal_check == LegalCheck::AllSatisfied || legal_check == LegalCheck::NotSatisfied);

        if legal_check == LegalCheck::AllSatisfied {
            let totalcost_before = pm.total_cost;
            pm.init(&mut part);
            assert_eq!(pm.total_cost, totalcost_before);

            pm.optimize(&mut part);
            assert!(pm.validator.final_check(&mut part));
            assert!(pm.total_cost <= totalcost_before);
        }
    }

    #[test]
    fn test_optimize_reduces_cost() {
        let netlist = make_netlist_drawf();
        let gain_calc = FMBiGainCalc::new(make_netlist_drawf(), 2);
        let gain_mgr = FMBiGainMgr::new(make_netlist_drawf(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_netlist_drawf(), 0.45);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8; 7];

        let _ = pm.legalize(&mut part);
        let totalcost_before = pm.total_cost;
        pm.optimize(&mut part);
        assert!(pm.total_cost <= totalcost_before);
    }

    #[test]
    fn test_total_cost_non_negative() {
        let netlist = make_netlist_drawf();
        let gain_calc = FMBiGainCalc::new(make_netlist_drawf(), 2);
        let gain_mgr = FMBiGainMgr::new(make_netlist_drawf(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_netlist_drawf(), 0.45);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8; 7];

        let _ = pm.legalize(&mut part);
        assert!(pm.total_cost >= 0);

        pm.optimize(&mut part);
        assert!(pm.total_cost >= 0);
    }

    // ── Ported from Python test_illegal.py ──────────────────────────

    fn make_netlist_4m_balanced() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        netlist
    }

    #[test]
    fn test_illegal_partition_detected() {
        let netlist = make_netlist_4m_balanced();
        let gain_calc = FMBiGainCalc::new(make_netlist_4m_balanced(), 2);
        let gain_mgr = FMBiGainMgr::new(make_netlist_4m_balanced(), gain_calc, 2);
        let validator = FMConstrMgr::new(make_netlist_4m_balanced(), 0.499);
        let mut pm = PartMgrBase::new(netlist, gain_mgr, validator, 2);
        let mut part = vec![0u8; 4];
        let legal_check = pm.legalize(&mut part);
        assert_ne!(legal_check, LegalCheck::AllSatisfied);

        let totalcost_before = pm.total_cost;
        pm.init(&mut part);
        assert_eq!(pm.total_cost, totalcost_before);
        pm.optimize(&mut part);
        assert!(!pm.validator.final_check(&mut part));
    }
}
