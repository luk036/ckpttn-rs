use std::collections::HashMap;
use std::collections::HashSet;

use crate::hypergraph::Hypergraph;
use crate::moveinfo::{MoveInfo, MoveInfoV};

/// Per-neighbour gain delta emitted by a [`GainCalcTrait`] update method.
///
/// Binary FM uses a single scalar: the delta is applied to the bucket of the
/// opposite partition (`1 - part_w`). K-way FM emits one entry per target
/// partition: `deltas[k]` is applied to bucket `k` for every `k != part_w`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GainDelta {
    /// Binary FM: a scalar delta applied to bucket `1 - part_w`.
    Scalar(i32),
    /// K-way FM: per-partition deltas, `deltas[k]` applied to bucket `k`.
    PerPart(Vec<i32>),
}

/// A bounded priority queue with integer keys that supports O(1) operations.
///
/// Uses a bucket array where each bucket is a `Vec<(key, node)>`. The node's
/// last known key is kept in `node_key` and persists after the node is popped
/// (mirroring `Dllink::data[0]` in the C++/Python reference), while `present`
/// tracks whether the node currently has a live entry. `locked` models the
/// per-link lock bit (`link.next is link`).
///
/// Deletion is lazy: updating a key pushes a new entry and leaves the old one
/// stale; [`BucketQueue::popleft_with_key`] and `refresh_max`
/// skip entries that are not present or whose stored key no longer matches
/// `node_key`.
///
/// Ported from C++ `BPQueue` in `bpqueue.hpp`.
pub struct BucketQueue<Node: Clone + Eq + std::hash::Hash> {
    buckets: Vec<Vec<(i32, Node)>>,
    offset: i32,
    node_key: HashMap<Node, i32>,
    present: HashSet<Node>,
    locked: HashSet<Node>,
    // Track current max key for O(1) popleft
    current_max: i32,
}

impl<Node: Clone + Eq + std::hash::Hash> BucketQueue<Node> {
    pub fn new(pmin: i32, pmax: i32) -> Self {
        let size = (pmax - pmin + 1).max(1) as usize;
        let mut buckets = Vec::with_capacity(size);
        for _ in 0..size {
            buckets.push(Vec::new());
        }
        BucketQueue {
            buckets,
            offset: pmin,
            node_key: HashMap::new(),
            present: HashSet::new(),
            locked: HashSet::new(),
            current_max: pmin - 1,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.present.is_empty()
    }

    #[inline]
    pub fn get_max(&mut self) -> i32 {
        self.refresh_max();
        self.current_max
    }

    /// The node's last known key (persists even when the node is not present).
    #[inline]
    pub fn get_key(&self, node: &Node) -> Option<i32> {
        self.node_key.get(node).copied()
    }

    #[inline]
    pub fn is_locked(&self, node: &Node) -> bool {
        self.locked.contains(node)
    }

    #[inline]
    fn bucket_index(&self, key: i32) -> Option<usize> {
        let idx = key - self.offset;
        if idx >= 0 && (idx as usize) < self.buckets.len() {
            Some(idx as usize)
        } else {
            None
        }
    }

    fn insert_raw(&mut self, key: i32, node: Node) {
        if let Some(idx) = self.bucket_index(key) {
            self.present.insert(node.clone());
            self.buckets[idx].push((key, node));
            if key > self.current_max {
                self.current_max = key;
            }
        } else {
            debug_assert!(
                false,
                "BucketQueue key {} out of range [{}, {}]",
                key,
                self.offset,
                self.offset + self.buckets.len() as i32 - 1
            );
        }
    }

    /// Push a node with the given absolute key (used at initialization).
    pub fn push(&mut self, key: i32, node: Node) {
        self.node_key.insert(node.clone(), key);
        self.insert_raw(key, node);
    }

    /// ADD `delta` to the node's current key and re-insert it.
    ///
    /// No-op if the node is locked in this queue. If the node has no stored key
    /// yet, the base key is 0 (mirroring the reference's `data[0]`).
    pub fn modify_key(&mut self, delta: i32, node: Node) {
        if self.locked.contains(&node) {
            return;
        }
        let base = self.node_key.get(&node).copied().unwrap_or(0);
        let new_key = base + delta;
        self.node_key.insert(node.clone(), new_key);
        self.insert_raw(new_key, node);
    }

    /// Write the node's key **without** making it present or inserting it.
    ///
    /// Mirrors `BPQueue::set_key` (only `link.data[0]` is written). The stored
    /// key is used as the base for a later [`BucketQueue::modify_key`].
    pub fn set_key(&mut self, key: i32, node: Node) {
        self.node_key.insert(node, key);
    }

    /// Pop the node with the highest key, skipping stale entries.
    pub fn popleft(&mut self) -> Option<Node> {
        self.popleft_with_key().map(|(node, _)| node)
    }

    /// Pop the highest-key fresh entry, returning the node and its true key.
    ///
    /// The returned key is the gain that must be reported for the move: stale
    /// entries can leave `current_max` too high, so callers must not use a
    /// separately cached maximum as the gain.
    pub fn popleft_with_key(&mut self) -> Option<(Node, i32)> {
        self.refresh_max();
        while self.current_max >= self.offset {
            let idx = (self.current_max - self.offset) as usize;
            if idx >= self.buckets.len() {
                break;
            }
            while let Some((stored_key, node)) = self.buckets[idx].pop() {
                let is_fresh = self.present.contains(&node)
                    && matches!(self.node_key.get(&node), Some(&k) if k == stored_key);
                if is_fresh {
                    self.present.remove(&node);
                    self.refresh_max();
                    return Some((node, stored_key));
                }
            }
            self.current_max -= 1;
        }
        None
    }

    fn refresh_max(&mut self) {
        while self.current_max >= self.offset {
            let idx = (self.current_max - self.offset) as usize;
            let fresh = if idx < self.buckets.len() {
                let node_key = &self.node_key;
                let present = &self.present;
                self.buckets[idx]
                    .iter()
                    .any(|(k, n)| present.contains(n) && node_key.get(n) == Some(k))
            } else {
                false
            };
            if fresh {
                return;
            }
            self.current_max -= 1;
        }
    }

    pub fn clear(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
        self.node_key.clear();
        self.present.clear();
        self.locked.clear();
        self.current_max = self.offset - 1;
    }

    #[inline]
    pub fn remove_node(&mut self, node: &Node) {
        self.present.remove(node);
    }

    #[inline]
    pub fn lock(&mut self, node: &Node) {
        self.present.remove(node);
        self.locked.insert(node.clone());
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
    pub(crate) nbrs_buf: Vec<Gnl::Node>,
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
            nbrs_buf: Vec::new(),
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        let total_cost = self.gain_calc.init(part);
        self.waiting_list.clear();
        // Clear buckets *and* per-link locks: the reference re-attaches every
        // link at init, which implicitly unlocks it (only fixed modules are
        // re-locked afterwards).
        for bucket in &mut self.gain_bucket {
            bucket.clear();
        }
        let modules: Vec<Gnl::Node> = self.hyprgraph.modules().collect();
        self.gain_calc
            .populate_buckets(part, &modules, &mut self.gain_bucket);
        total_cost
    }

    #[inline]
    pub fn is_empty_togo(&self, to_part: u8) -> bool {
        self.gain_bucket[to_part as usize].is_empty()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.gain_bucket.iter().all(|b| b.is_empty())
    }

    pub fn select(&mut self, part: &[u8]) -> (MoveInfoV<Gnl::Node>, i32) {
        let mut best_idx = 0usize;
        let mut best_max = self.gain_bucket[0].get_max();
        for i in 1..self.gain_bucket.len() {
            let m = self.gain_bucket[i].get_max();
            if m > best_max {
                best_max = m;
                best_idx = i;
            }
        }
        let to_part = best_idx as u8;
        if let Some((v, key)) = self.gain_bucket[best_idx].popleft_with_key() {
            let from_part = part[self.hyprgraph.module_index(v)];
            debug_assert_ne!(
                from_part, to_part,
                "selected a node already in the target partition"
            );
            return (
                MoveInfoV {
                    v,
                    from_part,
                    to_part,
                },
                key,
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
        if let Some((v, key)) = self.gain_bucket[to_part as usize].popleft_with_key() {
            return (v, key);
        }
        panic!("bucket {} empty in select_togo", to_part);
    }

    pub fn update_move(&mut self, part: &[u8], move_info_v: &MoveInfoV<Gnl::Node>)
    where
        GainCalc: GainCalcTrait<Gnl>,
    {
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

    /// Apply per-neighbour deltas, aligned with `gain_calc.idx_vec()`.
    fn apply_neighbor_deltas(&mut self, part: &[u8], deltas: Vec<GainDelta>) {
        let idx_vec = self.gain_calc.idx_vec().clone();
        for (i, delta) in deltas.into_iter().enumerate() {
            if let Some(&w) = idx_vec.get(i) {
                let part_w = part[self.hyprgraph.module_index(w)];
                self.modify_key(w, part_w, delta);
            }
        }
    }

    /// Lock a moved node.
    ///
    /// Binary FM has a single link per module, so locking it is global (all
    /// buckets). K-way FM locks only the `which_part` link, matching the
    /// reference; a vertex may still be re-selected from the other buckets.
    pub fn lock(&mut self, which_part: u8, v: Gnl::Node) {
        if self.num_parts == 2 {
            for bucket in &mut self.gain_bucket {
                bucket.lock(&v);
            }
        } else {
            self.gain_bucket[which_part as usize].lock(&v);
        }
    }

    pub fn lock_all(&mut self, _which_part: u8, v: Gnl::Node) {
        for bucket in &mut self.gain_bucket {
            bucket.lock(&v);
        }
    }

    pub fn update_move_v(&mut self, move_info_v: &MoveInfoV<Gnl::Node>, gain: i32) {
        let v = move_info_v.v;
        let from_part = move_info_v.from_part;
        let to_part = move_info_v.to_part;
        if self.num_parts == 2 {
            // Binary FM: the moved node is locked, so only record its reverse
            // gain (no re-insertion).
            self.gain_bucket[from_part as usize].set_key(-gain, v);
            return;
        }
        // K-way FM: adjust the moved node's gain in every remaining bucket,
        // then record the reverse gain for the source bucket.
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

    /// Apply a gain delta for a neighbour `w` currently in `part_w`.
    ///
    /// * `Scalar(d)` targets bucket `1 - part_w` (binary FM).
    /// * `PerPart(deltas)` targets every bucket `k != part_w` with `deltas[k]`.
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

/// Trait for gain calculators to be used with FMGainMgr.
pub trait GainCalcTrait<Gnl: Hypergraph> {
    fn init(&mut self, part: &[u8]) -> i32;
    fn update_move_init(&mut self);
    fn init_idx_vec(&mut self, v: Gnl::Node, net: Gnl::Node);
    fn idx_vec(&self) -> &Vec<Gnl::Node>;
    /// 2-pin update: returns the other endpoint and its gain delta.
    fn update_move_2pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> (Gnl::Node, GainDelta);
    /// 3-pin update: one gain delta per remaining endpoint (aligned with `idx_vec`).
    fn update_move_3pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<GainDelta>;
    /// General-net update: one gain delta per remaining endpoint.
    fn update_move_general_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<GainDelta>;
    /// K-way per-part deltas for the moved vertex; empty for binary FM.
    fn delta_gain_v(&self) -> &[i32];
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
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        assert_eq!(bq.get_max(), -6);
    }

    #[test]
    fn test_bucket_queue_popleft_empty() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        assert_eq!(bq.popleft(), None);
    }

    #[test]
    fn test_bucket_queue_modify_key_adds() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 10);
        bq.modify_key(3, 10);
        assert_eq!(bq.get_key(&10), Some(4));
        assert_eq!(bq.get_max(), 4);
        assert_eq!(bq.popleft(), Some(10));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_set_key_is_write_only() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.set_key(3, 42);
        assert!(bq.is_empty());
        assert_eq!(bq.get_key(&42), Some(3));
        bq.modify_key(2, 42);
        assert!(!bq.is_empty());
        assert_eq!(bq.get_max(), 5);
        assert_eq!(bq.popleft(), Some(42));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_lock_skips_modify() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 10);
        bq.lock(&10);
        assert!(bq.is_locked(&10));
        assert!(bq.is_empty());
        bq.modify_key(3, 10);
        assert!(bq.is_empty());
        assert_eq!(bq.popleft(), None);
    }

    #[test]
    fn test_bucket_queue_push_same_key() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(0, 10);
        bq.push(0, 20);
        let v1 = bq.popleft();
        assert!(v1 == Some(10) || v1 == Some(20));
    }

    #[test]
    fn test_bucket_queue_stale_entries_skipped() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(1, 42);
        bq.modify_key(3, 42);
        assert_eq!(bq.get_max(), 4);
        assert_eq!(bq.popleft(), Some(42));
        assert!(bq.is_empty());
    }

    #[test]
    fn test_bucket_queue_remove_node() {
        let mut bq: BucketQueue<i32> = BucketQueue::new(-5, 5);
        bq.push(3, 42);
        bq.remove_node(&42);
        assert_eq!(bq.popleft(), None);
        assert_eq!(bq.get_key(&42), Some(3));
    }

    use crate::test_support::make_nl;

    #[test]
    fn test_fm_gain_mgr_new() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        assert_eq!(mgr.num_parts, 2);
        assert!(mgr.waiting_list.is_empty());
    }

    #[test]
    fn test_fm_gain_mgr_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let cost = mgr.init(&part);
        assert_eq!(cost, 0);
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
        assert!(mgr.gain_bucket[0].is_locked(&nodes[0]));
        assert!(mgr.gain_bucket[1].is_locked(&nodes[0]));
    }

    #[test]
    fn test_fm_gain_mgr_lock_all() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        mgr.lock_all(0, nodes[1]);
        assert!(mgr.gain_bucket[0].is_locked(&nodes[1]));
        assert!(mgr.gain_bucket[1].is_locked(&nodes[1]));
    }

    #[test]
    fn test_fm_gain_mgr_modify_key_locked_node() {
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        mgr.lock(0, nodes[0]);
        mgr.modify_key(nodes[0], 0, GainDelta::Scalar(5));
        assert!(mgr.gain_bucket[1].is_empty());
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
        assert!(mgr.is_empty());
        assert_eq!(mgr.gain_bucket[0].get_key(&nodes[0]), Some(-5));
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
        assert!(!mgr.is_empty());
    }

    #[test]
    fn test_gain_mgr_select_after_init() {
        let netlist = make_nl();
        let calc = FMBiGainCalc::new(make_nl(), 2);
        let mut mgr: FMGainMgr<_, FMBiGainCalc<_>> = FMGainMgr::new(netlist, calc, 2);
        let part = vec![0u8, 0, 1, 1];
        let _ = mgr.init(&part);
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

        let mut iter = 0;
        while !mgr.is_empty() {
            iter += 1;
            assert!(iter < 1000, "gain manager failed to drain");
            let (move_info_v, gainmax) = mgr.select(&part);
            if gainmax <= 0 {
                continue;
            }
            mgr.lock(move_info_v.to_part, move_info_v.v);
            mgr.update_move(&part, &move_info_v);
            mgr.update_move_v(&move_info_v, gainmax);
            part[move_info_v.v.index()] = move_info_v.to_part;
        }
        assert!(part.iter().all(|&p| p == 0 || p == 1));
    }

    #[test]
    fn test_kway_lock_is_per_link() {
        use crate::fm_kway_gain_calc::FMKWayGainCalc;
        let netlist = make_nl();
        let calc = FMKWayGainCalc::new(make_nl(), 3);
        let mut mgr: FMGainMgr<_, FMKWayGainCalc<_>> = FMGainMgr::new(netlist, calc, 3);
        let nodes: Vec<NodeIndex> = make_nl().gr.node_indices().collect();
        mgr.lock(1, nodes[0]);
        assert!(mgr.gain_bucket[1].is_locked(&nodes[0]));
        assert!(!mgr.gain_bucket[0].is_locked(&nodes[0]));
        assert!(!mgr.gain_bucket[2].is_locked(&nodes[0]));
    }
}
