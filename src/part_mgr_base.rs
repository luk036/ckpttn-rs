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
