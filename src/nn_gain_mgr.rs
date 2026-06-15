use std::collections::HashSet;

use crate::fm_gain_mgr::{BucketQueue, GainCalcTrait};
use crate::hypergraph::Hypergraph;
use crate::moveinfo::{MoveInfo, MoveInfoV};

/// No-Nonsense Gain Manager for FM partitioning.
///
/// Implements gain management without a waiting list (unlike FMGainMgr).
/// Used with NNPartMgr for simpler direct-optimization partitioning passes.
/// Ported from Python `NNGainMgr` in `NNGainMgr.py`.
#[allow(dead_code)]
pub struct NNGainMgr<Gnl: Hypergraph, GainCalc> {
    pub gain_calc: GainCalc,
    hyprgraph: Gnl,
    pub gain_bucket: Vec<BucketQueue<Gnl::Node>>,
    pub num_parts: u8,
    locked_nodes: HashSet<usize>,
}

impl<Gnl: Hypergraph, GainCalc: GainCalcTrait<Gnl>> NNGainMgr<Gnl, GainCalc> {
    pub fn new(hyprgraph: Gnl, gain_calc: GainCalc, num_parts: u8) -> Self {
        let max_deg = hyprgraph.get_max_degree() as i32;
        let mut gain_bucket = Vec::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            gain_bucket.push(BucketQueue::new(-max_deg, max_deg));
        }
        NNGainMgr {
            gain_calc,
            hyprgraph,
            gain_bucket,
            num_parts,
            locked_nodes: HashSet::new(),
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        self.gain_calc.init(part)
    }

    pub fn is_empty(&self) -> bool {
        self.gain_bucket.iter().all(|b| b.is_empty())
    }

    pub fn is_empty_togo(&self, to_part: u8) -> bool {
        self.gain_bucket[to_part as usize].is_empty()
    }

    pub fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        let mut best_idx = 0;
        let mut best_max = self.gain_bucket[0].get_max();
        for (i, bucket) in self.gain_bucket.iter().enumerate().skip(1) {
            let m = bucket.get_max();
            if m > best_max {
                best_max = m;
                best_idx = i;
            }
        }
        let to_part = best_idx as u8;
        let gainmax = best_max;
        let v = self.gain_bucket[best_idx]
            .popleft()
            .expect("bucket should not be empty");
        let from_part = part[self.hyprgraph.module_index(v)];
        (
            MoveInfoV {
                v,
                from_part,
                to_part,
            },
            gainmax,
        )
    }

    pub fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32) {
        let gainmax = self.gain_bucket[to_part as usize].get_max();
        let v = self.gain_bucket[to_part as usize]
            .popleft()
            .expect("bucket should not be empty");
        (v, gainmax)
    }

    pub fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>) {
        self.gain_calc.update_move_init();
        let v = move_info_v.v;
        let nbrs: Vec<_> = self.hyprgraph.neighbors(v).collect();
        for net in nbrs {
            let degree = self.hyprgraph.degree(net);
            if !(2..=65536).contains(&degree) {
                continue;
            }
            let move_info = MoveInfo {
                net,
                v,
                from_part: move_info_v.from_part,
                to_part: move_info_v.to_part,
            };
            if degree == 2 {
                self.update_move_2pin_net(part, &move_info);
            } else {
                self.gain_calc.init_idx_vec(v, net);
                if degree == 3 {
                    self.update_move_3pin_net(part, &move_info);
                } else {
                    self.update_move_general_net(part, &move_info);
                }
            }
        }
    }

    fn update_move_2pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) {
        let w = self.gain_calc.update_move_2pin_net(part, move_info);
        let part_w = part[self.hyprgraph.module_index(w)];
        self.modify_key(w, part_w, self.gain_calc.delta_gain_w());
    }

    fn update_move_3pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) {
        let delta_gain = self.gain_calc.update_move_3pin_net(part, move_info);
        let idx_vec: Vec<_> = self.gain_calc.idx_vec().clone();
        for (i, &w) in idx_vec.iter().enumerate() {
            let dg = delta_gain[i];
            if dg != 0 {
                let part_w = part[self.hyprgraph.module_index(w)];
                self.modify_key(w, part_w, dg);
            }
        }
    }

    fn update_move_general_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) {
        let delta_gain = self.gain_calc.update_move_general_net(part, move_info);
        let idx_vec: Vec<_> = self.gain_calc.idx_vec().clone();
        for (i, &w) in idx_vec.iter().enumerate() {
            let dg = delta_gain[i];
            if dg != 0 {
                let part_w = part[self.hyprgraph.module_index(w)];
                self.modify_key(w, part_w, dg);
            }
        }
    }

    pub fn lock(&mut self, _which_part: u8, v: Gnl::Node) {
        self.locked_nodes.insert(self.hyprgraph.module_index(v));
    }

    pub fn lock_all(&mut self, _from_part: u8, v: Gnl::Node) {
        self.locked_nodes.insert(self.hyprgraph.module_index(v));
    }

    pub fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        self.set_key(move_info_v.from_part, move_info_v.v, -gain);
    }

    pub fn modify_key(&mut self, w: Gnl::Node, part_w: u8, key: i32) {
        if self.locked_nodes.contains(&self.hyprgraph.module_index(w)) {
            return;
        }
        self.set_key(part_w, w, key);
    }

    fn set_key(&mut self, which_part: u8, v: Gnl::Node, key: i32) {
        self.gain_bucket[which_part as usize].set_key(key, v);
    }
}

/// Trait for NN gain managers providing the interface used by NNPartMgr.
pub trait NNGainMgrInterface<Gnl: Hypergraph> {
    fn init(&mut self, part: &[u8]) -> i32;
    fn is_empty(&self) -> bool;
    fn is_empty_togo(&self, to_part: u8) -> bool;
    fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32);
    fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32);
    fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>);
    fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32);
    fn lock(&mut self, which_part: u8, v: Gnl::Node);
    fn modify_key(&mut self, w: Gnl::Node, part_w: u8, key: i32);
}

impl<Gnl: Hypergraph, GainCalc: GainCalcTrait<Gnl>> NNGainMgrInterface<Gnl>
    for NNGainMgr<Gnl, GainCalc>
{
    fn init(&mut self, part: &[u8]) -> i32 {
        self.init(part)
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    fn is_empty_togo(&self, to_part: u8) -> bool {
        self.is_empty_togo(to_part)
    }
    fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        self.select(part)
    }
    fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32) {
        self.select_togo(to_part)
    }
    fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>) {
        self.update_move(part, move_info_v)
    }
    fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        self.update_move_v(move_info_v, gain)
    }
    fn lock(&mut self, which_part: u8, v: Gnl::Node) {
        self.lock(which_part, v)
    }
    fn modify_key(&mut self, w: Gnl::Node, part_w: u8, key: i32) {
        self.modify_key(w, part_w, key)
    }
}
