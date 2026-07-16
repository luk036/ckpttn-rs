use std::collections::HashMap;
use std::collections::HashSet;

use crate::hypergraph::Hypergraph;
use crate::moveinfo::{MoveInfo, MoveInfoV};

/// A bounded priority queue with integer keys that supports O(1) operations.
///
/// Uses a bucket array where each bucket is a Vec<(key, node)>.
/// Maintains a HashMap from node to its current bucket index for O(1) key updates.
/// Ported from C++ `BPQueue` in `bpqueue.hpp`.
pub struct BucketQueue<Node: Clone + Eq + std::hash::Hash> {
    buckets: Vec<Vec<(i32, Node)>>,
    offset: i32,
    node_bucket: HashMap<Node, i32>,
    // Track current max key for O(1) popleft
    current_max: i32,
}

impl<Node: Clone + Eq + std::hash::Hash> BucketQueue<Node> {
    pub fn new(pmin: i32, pmax: i32) -> Self {
        let size = (pmax - pmin + 1) as usize;
        let mut buckets = Vec::with_capacity(size);
        for _ in 0..size {
            buckets.push(Vec::new());
        }
        BucketQueue {
            buckets,
            offset: pmin,
            node_bucket: HashMap::new(),
            current_max: pmin - 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.current_max < self.offset || self.node_bucket.is_empty()
    }

    pub fn get_max(&self) -> i32 {
        self.current_max
    }

    /// Push a node with the given key into the queue.
    /// If the node already exists, the old entry stays (stale).
    /// The node's position is tracked in node_bucket.
    pub fn push(&mut self, key: i32, node: Node) {
        let idx = (key - self.offset) as usize;
        if idx < self.buckets.len() {
            // Track the node's current key
            self.node_bucket.insert(node.clone(), key);
            self.buckets[idx].push((key, node));
            if key > self.current_max {
                self.current_max = key;
            }
        }
    }

    /// Pop the node with the highest key, skipping stale entries.
    /// A stale entry is one whose key doesn't match the node's tracked key.
    pub fn popleft(&mut self) -> Option<Node> {
        self.refresh_max();
        while self.current_max >= self.offset {
            let idx = (self.current_max - self.offset) as usize;
            if idx >= self.buckets.len() {
                break;
            }
            while let Some((stored_key, node)) = self.buckets[idx].pop() {
                let is_fresh = match self.node_bucket.get(&node) {
                    Some(&k) if k == stored_key => true,
                    _ => false,
                };
                if is_fresh {
                    self.node_bucket.remove(&node);
                    // Refresh max after removal
                    self.refresh_max();
                    return Some(node);
                }
            }
            self.current_max -= 1;
            self.refresh_max();
        }
        None
    }

    fn refresh_max(&mut self) {
        while self.current_max >= self.offset {
            let idx = (self.current_max - self.offset) as usize;
            if idx < self.buckets.len() && !self.buckets[idx].is_empty() {
                return;
            }
            if self.current_max < self.offset {
                break;
            }
            self.current_max -= 1;
        }
    }

    /// Modify the key of a node: push a new entry with the given key.
    /// The old entry becomes stale and will be skipped by popleft.
    pub fn modify_key(&mut self, key: i32, node: Node) {
        self.node_bucket.insert(node.clone(), key);
        let idx = (key - self.offset) as usize;
        if idx < self.buckets.len() {
            self.buckets[idx].push((key, node));
            if key > self.current_max {
                self.current_max = key;
            }
        }
    }

    /// Set key: same as modify_key (just push new entry, old becomes stale).
    pub fn set_key(&mut self, key: i32, node: Node) {
        self.modify_key(key, node);
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.node_bucket.clear();
        self.current_max = self.offset - 1;
    }

    /// Remove a specific node from tracking (used when node is locked/moved).
    pub fn remove_node(&mut self, node: &Node) {
        self.node_bucket.remove(node);
    }
}

/// Fiduccia-Mattheyses Gain Manager
///
/// Base class for managing gain calculation and bucket structure.
/// Ported from C++ `FMGainMgr` in `FMGainMgr.hpp`/`FMGainMgr.cpp`.
#[allow(dead_code)]
pub struct FMGainMgr<Gnl: Hypergraph, GainCalc> {
    pub gain_calc: GainCalc,
    pub(crate) waiting_list: Vec<Gnl::Node>,
    pub(crate) hyprgraph: Gnl,
    pub(crate) gain_bucket: Vec<BucketQueue<Gnl::Node>>,
    pub(crate) num_parts: u8,
    pub(crate) locked_nodes: HashSet<usize>,
}

impl<Gnl: Hypergraph, GainCalc> FMGainMgr<Gnl, GainCalc>
where
    GainCalc: GainCalcTrait<Gnl>,
{
    pub fn new(hyprgraph: Gnl, gain_calc: GainCalc, num_parts: u8) -> Self {
        let max_deg = hyprgraph.get_max_degree() as i32;
        let range = (num_parts as i32 - 1) * max_deg;
        let mut gain_bucket = Vec::with_capacity(num_parts as usize);
        for _ in 0..num_parts {
            gain_bucket.push(BucketQueue::new(-range, range));
        }
        FMGainMgr {
            gain_calc,
            waiting_list: Vec::new(),
            hyprgraph,
            gain_bucket,
            num_parts,
            locked_nodes: HashSet::new(),
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        let total_cost = self.gain_calc.init(part);
        self.waiting_list.clear();
        // Populate gain buckets from the gain calculator's initial gains
        let modules: Vec<Gnl::Node> = self.hyprgraph.modules().collect();
        self.gain_calc
            .populate_buckets(part, &modules, &mut self.gain_bucket);
        total_cost
    }

    pub fn is_empty_togo(&self, to_part: u8) -> bool {
        self.gain_bucket[to_part as usize].is_empty()
    }

    pub fn is_empty(&self) -> bool {
        self.gain_bucket.iter().all(|b| b.is_empty())
    }

    pub fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        // Find the bucket with the highest max gain
        let mut best_idx = 0usize;
        let mut best_max = self.gain_bucket[0].get_max();
        for (i, bucket) in self.gain_bucket.iter().enumerate().skip(1) {
            let m = bucket.get_max();
            if m > best_max {
                best_max = m;
                best_idx = i;
            }
        }
        // Try to pop from the best bucket
        let to_part = best_idx as u8;
        if let Some(v) = self.gain_bucket[best_idx].popleft() {
            let from_part = part[self.hyprgraph.module_index(v)];
            return (
                MoveInfoV {
                    v,
                    from_part,
                    to_part,
                },
                best_max,
            );
        }
        let sentinel_gain = -i32::MAX + 1;
        let v = self.hyprgraph.modules().next().unwrap();
        let from_part = part[self.hyprgraph.module_index(v)];
        (
            MoveInfoV {
                v,
                from_part,
                to_part,
            },
            sentinel_gain,
        )
    }

    pub fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32) {
        let gainmax = self.gain_bucket[to_part as usize].get_max();
        if let Some(v) = self.gain_bucket[to_part as usize].popleft() {
            return (v, gainmax);
        }
        panic!("bucket {} empty in select_togo", to_part);
    }

    pub fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>)
    where
        GainCalc: GainCalcTrait<Gnl>,
    {
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
        // Remove node from bucket tracking so stale entries are skipped
        for bucket in &mut self.gain_bucket {
            bucket.remove_node(&v);
        }
    }

    pub fn lock_all(&mut self, _from_part: u8, v: Gnl::Node) {
        self.locked_nodes.insert(self.hyprgraph.module_index(v));
        for bucket in &mut self.gain_bucket {
            bucket.remove_node(&v);
        }
    }

    pub fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        // Only update key if node is not locked (locked nodes were already moved)
        if !self
            .locked_nodes
            .contains(&self.hyprgraph.module_index(move_info_v.v))
        {
            self._set_key(move_info_v.from_part, move_info_v.v, -gain);
        }
    }

    pub fn modify_key(&mut self, w: Gnl::Node, part_w: u8, key: i32) {
        if self.locked_nodes.contains(&self.hyprgraph.module_index(w)) {
            return;
        }
        self._set_key(part_w, w, key);
    }

    fn _set_key(&mut self, which_part: u8, v: Gnl::Node, key: i32) {
        self.gain_bucket[which_part as usize].set_key(key, v);
    }
}

/// Trait for gain managers providing the interface used by PartMgrBase.
pub trait GainMgrInterface<Gnl: Hypergraph> {
    fn init(&mut self, part: &[u8]) -> i32;
    fn is_empty(&self) -> bool;
    fn is_empty_togo(&self, to_part: u8) -> bool;
    fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32);
    fn select_togo(&mut self, to_part: u8) -> (Gnl::Node, i32);
    fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>);
    fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32);
    fn lock(&mut self, which_part: u8, v: Gnl::Node);
}

/// Trait for constraint managers providing the interface used by PartMgrBase.
pub trait ConstrMgrInterface<Gnl: Hypergraph> {
    fn init(&mut self, part: &[u8]);
    fn check_legal(
        &mut self,
        move_info_v: &MoveInfoV<Gnl::Node>,
    ) -> crate::fm_constr_mgr::LegalCheck;
    fn check_constraints(&self, move_info_v: &MoveInfoV<Gnl::Node>) -> bool;
    fn update_move(&mut self, move_info_v: &MoveInfoV<Gnl::Node>);
    fn select_togo(&self) -> u8;
    fn final_check(&mut self, part: &[u8]) -> bool;
}

impl<Gnl: Hypergraph, GainCalc: GainCalcTrait<Gnl>> GainMgrInterface<Gnl>
    for FMGainMgr<Gnl, GainCalc>
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
}

/// Trait for gain calculators to be used with FMGainMgr.
pub trait GainCalcTrait<Gnl: Hypergraph> {
    fn init(&mut self, part: &[u8]) -> i32;
    fn update_move_init(&mut self);
    fn init_idx_vec(&mut self, v: Gnl::Node, net: Gnl::Node);
    fn idx_vec(&self) -> &Vec<Gnl::Node>;
    fn update_move_2pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) -> Gnl::Node;
    fn update_move_3pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) -> Vec<i32>;
    fn update_move_general_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>)
        -> Vec<i32>;
    fn delta_gain_w(&self) -> i32;
    /// Populate gain buckets from initial gain values after init().
    fn populate_buckets(
        &self,
        part: &[u8],
        modules: &[Gnl::Node],
        gain_bucket: &mut [BucketQueue<Gnl::Node>],
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fm_bi_gain_calc::FMBiGainCalc;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    #[test]
    fn test_bucket_queue() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        assert!(bq.is_empty());
        bq.push(3, 42);
        assert!(!bq.is_empty());
        assert_eq!(bq.get_max(), 3);
        assert_eq!(bq.popleft(), Some(42));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_multiple() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 10);
        bq.push(5, 50);
        bq.push(3, 30);
        assert_eq!(bq.get_max(), 5);
        assert_eq!(bq.popleft(), Some(50));
        assert_eq!(bq.get_max(), 3);
        assert_eq!(bq.popleft(), Some(30));
        assert_eq!(bq.popleft(), Some(10));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_get_max_empty() {
        let bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        assert_eq!(bq.get_max(), -6);
    }

    #[test]
    fn test_bucket_queue_popleft_empty() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        assert_eq!(bq.popleft(), None);
    }

    #[test]
    fn test_bucket_queue_modify_key() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 10);
        bq.modify_key(5, 10);
        // Old entry (key=1) is stale, should be skipped
        assert_eq!(bq.get_max(), 5);
        assert_eq!(bq.popleft(), Some(10));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_set_key() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.set_key(3, 42);
        assert_eq!(bq.popleft(), Some(42));
    }

    #[test]
    fn test_bucket_queue_push_same_key() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(0, 10);
        bq.push(0, 20);
        // The second push for key 0 overwrites the tracking entry for node 20,
        // but node 10's entry is still tracked. popleft returns the last pushed
        // (since we pop() from Vec). Both have key=0.
        let v1 = bq.popleft();
        assert!(v1 == Some(10) || v1 == Some(20));
    }

    #[test]
    fn test_bucket_queue_stale_entries_skipped() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 42); // initial entry
        bq.modify_key(3, 42); // updated entry, old one is stale
                              // Should skip stale (key=1) and return fresh (key=3)
        assert_eq!(bq.get_max(), 3);
        assert_eq!(bq.popleft(), Some(42));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_remove_node() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(3, 42);
        bq.remove_node(&42);
        // Node 42 is removed from tracking, so popleft should skip it
        assert_eq!(bq.popleft(), None);
    }

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
    fn test_fm_gain_mgr_new() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert_eq!(mgr.num_parts, 2);
        assert!(mgr.waiting_list.is_empty());
        assert!(mgr.locked_nodes.is_empty());
    }

    #[test]
    fn test_fm_gain_mgr_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = mgr.init(&part);
        assert_eq!(cost, 0);
        // After init, buckets should NOT be empty (they should have nodes)
        assert!(!mgr.is_empty());
    }

    #[test]
    fn test_fm_gain_mgr_is_empty_togo() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert!(mgr.is_empty_togo(0));
        assert!(mgr.is_empty_togo(1));
    }

    #[test]
    fn test_fm_gain_mgr_is_empty() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_fm_gain_mgr_lock() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        mgr.lock(0, nodes[0]);
        assert!(mgr.locked_nodes.contains(&0));
    }

    #[test]
    fn test_fm_gain_mgr_lock_all() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        mgr.lock_all(0, nodes[1]);
        assert!(mgr.locked_nodes.contains(&1));
    }

    #[test]
    fn test_fm_gain_mgr_modify_key_locked_node() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        mgr.lock(0, nodes[0]);
        mgr.modify_key(nodes[0], 0, 5);
    }

    #[test]
    fn test_fm_gain_mgr_update_move_v() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let move_info_v = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        mgr.update_move_v(&move_info_v, 5);
    }

    #[test]
    fn test_gain_mgr_interface_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = GainMgrInterface::init(&mut mgr, &part);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_gain_mgr_interface_is_empty() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert!(GainMgrInterface::is_empty(&mgr));
    }

    #[test]
    fn test_gain_mgr_interface_is_empty_togo() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert!(GainMgrInterface::is_empty_togo(&mgr, 0));
    }

    #[test]
    fn test_gain_mgr_interface_lock() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        GainMgrInterface::lock(&mut mgr, 0, nodes[0]);
    }

    #[test]
    fn test_gain_mgr_interface_update_move_v() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let move_info_v = MoveInfoV {
            v: nodes[0],
            from_part: 0,
            to_part: 1,
        };
        GainMgrInterface::update_move_v(&mut mgr, &move_info_v, 3);
    }

    #[test]
    fn test_gain_mgr_init_then_non_empty() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = mgr.init(&part);
        assert_eq!(cost, 0);
        // After init with populated buckets, should NOT be empty
        assert!(!mgr.is_empty());
    }

    #[test]
    fn test_gain_mgr_select_after_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let _ = mgr.init(&part);
        // After init with populated buckets, select should return a valid move
        // with from_part != to_part
        let (move_info_v, _gain) = mgr.select(&part);
        assert_eq!(move_info_v.to_part, 1 - move_info_v.from_part);
    }

    #[test]
    fn test_full_gain_mgr_flow() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let mut part = vec![0u8, 0, 1, 1];
        let _ = mgr.init(&part);

        while !mgr.is_empty() {
            let (move_info_v, gainmax) = mgr.select(&part);
            if gainmax <= 0 {
                continue;
            }
            mgr.update_move(&part, &move_info_v);
            mgr.update_move_v(&move_info_v, gainmax);
            part[move_info_v.v.index()] = move_info_v.to_part;
        }
        assert!(part.iter().all(|&p| p == 0 || p == 1));
    }
}
