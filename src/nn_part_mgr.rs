use crate::fm_constr_mgr::LegalCheck;
use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfoV;
use crate::nn_gain_mgr::NNGainMgrInterface;

/// No-Nonsense Partition Manager.
///
/// Implements a simplified FM-style partition optimization without
/// backtracking (no snapshot/restore). Unlike PartMgrBase, optimize_1pass
/// only accepts positive-gain moves and stops at the first non-improving move.
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
    GainMgr: NNGainMgrInterface<Gnl>,
    ConstrMgr: super::fm_gain_mgr::ConstrMgrInterface<Gnl>,
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
            self.gain_mgr.update_move_v(&move_info_v, gainmax);
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
        let mut totalgain = 0i32;

        while !self.gain_mgr.is_empty() {
            let (move_info_v, gainmax) = self.gain_mgr.select(part);
            if gainmax <= 0 {
                break;
            }
            let satisfied_ok = self.validator.check_constraints(&move_info_v);
            if !satisfied_ok {
                continue;
            }
            self.gain_mgr.update_move(part, &move_info_v);
            self.gain_mgr.update_move_v(&move_info_v, gainmax);
            self.validator.update_move(&move_info_v);
            totalgain += gainmax;
            part[self.hyprgraph.module_index(move_info_v.v)] = move_info_v.to_part;
        }
        self.total_cost -= totalgain;
    }

    pub fn final_check(&mut self, part: &mut [u8]) -> bool {
        self.validator.final_check(part)
    }
}
