use crate::fm_gain_mgr::{BucketQueue, GainCalcTrait, GainDelta, GainMgrInterface};
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
    nbrs_buf: Vec<Gnl::Node>,
}

impl<Gnl: Hypergraph, GainCalc: GainCalcTrait<Gnl>> NNGainMgr<Gnl, GainCalc> {
    pub fn new(hyprgraph: Gnl, gain_calc: GainCalc, num_parts: u8) -> Self {
        let max_deg = hyprgraph.get_max_degree() as i32;
        let range = (num_parts as i32 - 1) * max_deg;
        let mut gain_bucket = Vec::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            gain_bucket.push(BucketQueue::new(-range, range));
        }
        NNGainMgr {
            gain_calc,
            hyprgraph,
            gain_bucket,
            num_parts,
            nbrs_buf: Vec::new(),
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        let total_cost = self.gain_calc.init(part);
        for bucket in &mut self.gain_bucket {
            bucket.clear();
        }
        let modules: Vec<Gnl::Node> = self.hyprgraph.modules().collect();
        self.gain_calc
            .populate_buckets(part, &modules, &mut self.gain_bucket);
        total_cost
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.gain_bucket.iter().all(|b| b.is_empty())
    }

    #[inline]
    pub fn is_empty_togo(&self, to_part: u8) -> bool {
        self.gain_bucket[to_part as usize].is_empty()
    }

    pub fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        let mut best_idx = 0;
        let mut best_max = self.gain_bucket[0].get_max();
        for i in 1..self.gain_bucket.len() {
            let m = self.gain_bucket[i].get_max();
            if m > best_max {
                best_max = m;
                best_idx = i;
            }
        }
        let to_part = best_idx as u8;
        let (v, gainmax) = self.gain_bucket[best_idx]
            .popleft_with_key()
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
        let (v, gainmax) = self.gain_bucket[to_part as usize]
            .popleft_with_key()
            .expect("bucket should not be empty");
        (v, gainmax)
    }

    pub fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>) {
        self.gain_calc.update_move_init();
        let v = move_info_v.v;
        self.nbrs_buf.clear();
        self.nbrs_buf.extend(self.hyprgraph.neighbors(v));
        let num_nbrs = self.nbrs_buf.len();
        for i in 0..num_nbrs {
            let net = self.nbrs_buf[i];
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
        let (w, delta) = self.gain_calc.update_move_2pin_net(part, move_info);
        let part_w = part[self.hyprgraph.module_index(w)];
        self.modify_key(w, part_w, delta);
    }

    fn update_move_3pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) {
        let deltas = self.gain_calc.update_move_3pin_net(part, move_info);
        self.apply_neighbor_deltas(part, deltas);
    }

    fn update_move_general_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) {
        let deltas = self.gain_calc.update_move_general_net(part, move_info);
        self.apply_neighbor_deltas(part, deltas);
    }

    fn apply_neighbor_deltas(&mut self, part: &[u8], deltas: Vec<GainDelta>) {
        let idx_vec = self.gain_calc.idx_vec().clone();
        for (i, delta) in deltas.into_iter().enumerate() {
            if let Some(&w) = idx_vec.get(i) {
                let part_w = part[self.hyprgraph.module_index(w)];
                self.modify_key(w, part_w, delta);
            }
        }
    }

    #[inline]
    pub fn lock(&mut self, which_part: u8, v: Gnl::Node) {
        if self.num_parts == 2 {
            for bucket in &mut self.gain_bucket {
                bucket.lock(&v);
            }
        } else {
            self.gain_bucket[which_part as usize].lock(&v);
        }
    }

    #[inline]
    pub fn lock_all(&mut self, _which_part: u8, v: Gnl::Node) {
        for bucket in &mut self.gain_bucket {
            bucket.lock(&v);
        }
    }

    #[inline]
    pub fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        let v = move_info_v.v;
        let from_part = move_info_v.from_part;
        let to_part = move_info_v.to_part;
        if self.num_parts == 2 {
            self.gain_bucket[from_part as usize].set_key(-gain, v);
            return;
        }
        let deltas: Vec<i32> = self.gain_calc.delta_gain_v().to_vec();
        for k in 0..self.num_parts as usize {
            if k != from_part as usize && k != to_part as usize {
                let d = deltas.get(k).copied().unwrap_or(0);
                if d != 0 {
                    self.gain_bucket[k].modify_key(d, v);
                }
            }
        }
        self.gain_bucket[from_part as usize].set_key(-gain, v);
    }

    pub fn modify_key(&mut self, w: Gnl::Node, part_w: u8, delta: GainDelta) {
        match delta {
            GainDelta::Scalar(d) => {
                if d != 0 {
                    let dest = (1 - part_w) as usize;
                    self.gain_bucket[dest].modify_key(d, w);
                }
            }
            GainDelta::PerPart(deltas) => {
                for k in 0..self.num_parts as usize {
                    if k != part_w as usize {
                        let d = deltas.get(k).copied().unwrap_or(0);
                        if d != 0 {
                            self.gain_bucket[k].modify_key(d, w);
                        }
                    }
                }
            }
        }
    }
}

impl<Gnl: Hypergraph, GainCalc: GainCalcTrait<Gnl>> GainMgrInterface<Gnl>
    for NNGainMgr<Gnl, GainCalc>
{
    #[inline]
    fn init(&mut self, part: &[u8]) -> i32 {
        self.init(part)
    }
    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
    #[inline]
    fn is_empty_togo(&self, to_part: u8) -> bool {
        self.is_empty_togo(to_part)
    }
    #[inline]
    fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        self.select(part)
    }
    #[inline]
    fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32) {
        self.select_togo(to_part)
    }
    #[inline]
    fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>) {
        self.update_move(part, move_info_v)
    }
    #[inline]
    fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        self.update_move_v(move_info_v, gain)
    }
    #[inline]
    fn lock(&mut self, which_part: u8, v: Gnl::Node) {
        self.lock(which_part, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fm_bi_gain_calc::FMBiGainCalc;
    use crate::hypergraph::SimpleNetlist;
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
    fn test_nn_gain_mgr_new() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);
        assert_eq!(mgr.num_parts, 2);
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_nn_gain_mgr_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = mgr.init(&part);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_nn_gain_mgr_is_empty_togo() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);
        assert!(mgr.is_empty_togo(0));
        assert!(mgr.is_empty_togo(1));
    }

    #[test]
    fn test_nn_gain_mgr_lock_and_lock_all() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);
        mgr.lock(0, nodes[0]);
        assert!(mgr.gain_bucket[0].is_locked(&nodes[0]));
        assert!(mgr.gain_bucket[1].is_locked(&nodes[0]));
        mgr.lock_all(0, nodes[1]);
        assert!(mgr.gain_bucket[0].is_locked(&nodes[1]));
        assert!(mgr.gain_bucket[1].is_locked(&nodes[1]));
    }

    #[test]
    fn test_nn_gain_mgr_update_move_v_and_modify_key() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);
        let move_info_v = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        mgr.update_move_v(&move_info_v, 5);
        assert!(mgr.is_empty());
        mgr.modify_key(nodes[0], 0, GainDelta::Scalar(1));
        assert!(!mgr.is_empty());
    }

    #[test]
    fn test_nn_gain_mgr_interface_trait() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: NNGainMgr<_, FMBiGainCalc<_>> = NNGainMgr::new(netlist, calc, 2);

        let part = vec![0u8, 0, 1, 1];
        let cost = GainMgrInterface::init(&mut mgr, &part);
        assert_eq!(cost, 0);
        assert!(!GainMgrInterface::is_empty(&mgr));
        assert!(!GainMgrInterface::is_empty_togo(&mgr, 0));
        assert!(!GainMgrInterface::is_empty_togo(&mgr, 1));

        GainMgrInterface::lock(&mut mgr, 0, nodes[0]);
        let move_info_v = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        GainMgrInterface::update_move_v(&mut mgr, &move_info_v, 3);
    }
}
