use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfo;

/// K-way gain calculator for FM partitioning.
///
/// Supports N partitions with per-partition gain tracking.
/// Ported from Python `FMKWayGainCalc` in `FMKWayGainCalc.py`.
pub struct FMKWayGainCalc<Gnl: Hypergraph> {
    pub(crate) hyprgraph: Gnl,
    pub init_gain_matrix: Vec<Vec<i32>>,
    pub total_cost: i32,
    pub delta_gain_v: Vec<i32>,
    pub idx_vec: Vec<Gnl::Node>,
    pub delta_gain_w: Vec<Vec<i32>>,
    pub num_parts: u8,
}

impl<Gnl: Hypergraph> FMKWayGainCalc<Gnl> {
    pub fn new(hyprgraph: Gnl, num_parts: u8) -> Self {
        let nmod = hyprgraph.number_of_modules();
        let init_gain_matrix = vec![vec![0i32; nmod]; num_parts as usize];
        FMKWayGainCalc {
            hyprgraph,
            init_gain_matrix,
            total_cost: 0,
            delta_gain_v: Vec::new(),
            idx_vec: Vec::new(),
            delta_gain_w: Vec::new(),
            num_parts,
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        self.total_cost = 0;
        for row in &mut self.init_gain_matrix {
            for elem in row.iter_mut() {
                *elem = 0;
            }
        }
        let nets: Vec<_> = self.hyprgraph.nets().collect();
        for net in nets {
            self.init_gain(net, part);
        }
        self.total_cost
    }

    pub fn update_move_init(&mut self) {
        self.delta_gain_v = vec![0; self.num_parts as usize];
    }

    pub fn init_idx_vec(&mut self, v: Gnl::Node, net: Gnl::Node) {
        self.idx_vec.clear();
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        self.idx_vec.reserve(nbrs.len() - 1);
        for w in nbrs {
            if w != v {
                self.idx_vec.push(w);
            }
        }
    }

    pub fn idx_vec(&self) -> &Vec<Gnl::Node> {
        &self.idx_vec
    }

    fn module_idx(&self, v: Gnl::Node) -> usize {
        self.hyprgraph.module_index(v)
    }

    fn modify_gain(&mut self, v: Gnl::Node, pv: u8, weight: i32) {
        let idx = self.hyprgraph.module_index(v);
        for k in 0..self.num_parts {
            if k != pv {
                self.init_gain_matrix[k as usize][idx] += weight;
            }
        }
    }

    fn init_gain(&mut self, net: Gnl::Node, part: &[u8]) {
        let degree = self.hyprgraph.degree(net);
        if !(2..=65536).contains(&degree) {
            return;
        }
        match degree {
            2 => self.init_gain_2pin_net(net, part),
            3 => self.init_gain_3pin_net(net, part),
            _ => self.init_gain_general_net(net, part),
        }
    }

    fn init_gain_2pin_net(&mut self, net: Gnl::Node, part: &[u8]) {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let w = nbrs[0];
        let v = nbrs[1];
        let weight = self.hyprgraph.get_net_weight(net) as i32;
        let i_w = self.hyprgraph.module_index(w);
        let i_v = self.hyprgraph.module_index(v);
        let part_w = part[i_w];
        let part_v = part[i_v];
        if part_v == part_w {
            for &a in &[w, v] {
                let i_a = self.hyprgraph.module_index(a);
                self.modify_gain(a, part[i_a], -weight);
            }
        } else {
            self.total_cost += weight;
            self.init_gain_matrix[part_v as usize][i_w] += weight;
            self.init_gain_matrix[part_w as usize][i_v] += weight;
        }
    }

    fn init_gain_3pin_net(&mut self, net: Gnl::Node, part: &[u8]) {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let w = nbrs[0];
        let v = nbrs[1];
        let u = nbrs[2];
        let weight = self.hyprgraph.get_net_weight(net) as i32;
        let i_u = self.hyprgraph.module_index(u);
        let i_v = self.hyprgraph.module_index(v);
        let i_w = self.hyprgraph.module_index(w);
        let pu = part[i_u];
        let pv = part[i_v];
        let pw = part[i_w];

        let _do_modify_gain = |this: &mut Self, node: Gnl::Node, p: u8, w: i32| {
            this.modify_gain(node, p, w);
        };

        if pu == pv {
            if pw == pv {
                for &a in &[u, v, w] {
                    let i_a = self.hyprgraph.module_index(a);
                    self.modify_gain(a, part[i_a], -weight);
                }
                return;
            }
            let (a, b, c) = (w, u, v);
            let i_a = self.hyprgraph.module_index(a);
            let i_b = self.hyprgraph.module_index(b);
            self.init_gain_matrix[part[i_b] as usize][i_a] += weight;
            for &e in &[b, c] {
                let i_e = self.hyprgraph.module_index(e);
                self.modify_gain(e, part[i_e], -weight);
                self.init_gain_matrix[part[i_a] as usize][i_e] += weight;
            }
            self.total_cost += weight;
        } else if pw == pv {
            let (a, b, c) = (u, v, w);
            let i_a = self.hyprgraph.module_index(a);
            let i_b = self.hyprgraph.module_index(b);
            self.init_gain_matrix[part[i_b] as usize][i_a] += weight;
            for &e in &[b, c] {
                let i_e = self.hyprgraph.module_index(e);
                self.modify_gain(e, part[i_e], -weight);
                self.init_gain_matrix[part[i_a] as usize][i_e] += weight;
            }
            self.total_cost += weight;
        } else if pw == pu {
            let (a, b, c) = (v, w, u);
            let i_a = self.hyprgraph.module_index(a);
            let i_b = self.hyprgraph.module_index(b);
            self.init_gain_matrix[part[i_b] as usize][i_a] += weight;
            for &e in &[b, c] {
                let i_e = self.hyprgraph.module_index(e);
                self.modify_gain(e, part[i_e], -weight);
                self.init_gain_matrix[part[i_a] as usize][i_e] += weight;
            }
            self.total_cost += weight;
        } else {
            self.total_cost += 2 * weight;
            let nodes = [u, v, w];
            for &a in &nodes {
                let i_a = self.hyprgraph.module_index(a);
                let pa = part[i_a] as usize;
                for &b in &nodes {
                    if a != b {
                        let i_b = self.hyprgraph.module_index(b);
                        self.init_gain_matrix[pa][i_b] += weight;
                    }
                }
            }
        }
    }

    fn init_gain_general_net(&mut self, net: Gnl::Node, part: &[u8]) {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let mut num = vec![0usize; self.num_parts as usize];
        for &w in &nbrs {
            let i_w = self.hyprgraph.module_index(w);
            let p = part[i_w] as usize;
            if p < self.num_parts as usize {
                num[p] += 1;
            }
        }
        let weight = self.hyprgraph.get_net_weight(net) as i32;

        for c in &num {
            if *c > 0 {
                self.total_cost += weight;
            }
        }
        self.total_cost -= weight;

        for (k, &c) in num.iter().enumerate() {
            if c == 0 {
                for &w in &nbrs {
                    let i_w = self.hyprgraph.module_index(w);
                    self.init_gain_matrix[k][i_w] -= weight;
                }
            } else if c == 1 {
                for &w in &nbrs {
                    if part[self.module_idx(w)] as usize == k {
                        self.modify_gain(w, k as u8, weight);
                        break;
                    }
                }
            }
        }
    }

    pub fn update_move_2pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Gnl::Node {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(move_info.net).collect();
        let first = nbrs[0];
        let second = nbrs[1];
        let w = if first != move_info.v { first } else { second };
        let part_w = part[self.hyprgraph.module_index(w)];
        let mut weight = self.hyprgraph.get_net_weight(move_info.net) as i32;
        let fp = move_info.from_part;
        let tp = move_info.to_part;

        let mut dg_w = vec![0i32; self.num_parts as usize];
        for &l_part in &[fp, tp] {
            if part_w == l_part {
                for (k, dg_w_elem) in dg_w.iter_mut().enumerate() {
                    *dg_w_elem += weight;
                    self.delta_gain_v[k] += weight;
                }
            }
            dg_w[l_part as usize] -= weight;
            weight = -weight;
        }

        self.delta_gain_w = vec![dg_w];
        w
    }

    pub fn update_move_3pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<Vec<i32>> {
        let (_, fp, tp) = (move_info.v, move_info.from_part, move_info.to_part);
        let weight = self.hyprgraph.get_net_weight(move_info.net) as i32;
        let nparts = self.num_parts as usize;

        let mut delta_gain = vec![vec![0i32; nparts]; self.idx_vec.len()];

        let part_w = part[self.hyprgraph.module_index(self.idx_vec[0])];
        let part_u = part[self.hyprgraph.module_index(self.idx_vec[1])];

        if part_w == part_u {
            let mut cur_fp = fp;
            let mut cur_tp = tp;
            for _ in 0..2 {
                if part_w != cur_fp {
                    delta_gain[0][cur_fp as usize] -= weight;
                    delta_gain[1][cur_fp as usize] -= weight;
                    if part_w == cur_tp {
                        for k in 0..nparts {
                            self.delta_gain_v[k] -= weight;
                        }
                    }
                }
                std::mem::swap(&mut cur_fp, &mut cur_tp);
            }
            return delta_gain;
        }

        let mut cur_fp = fp;
        let mut cur_tp = tp;
        for _ in 0..2 {
            if part_w == cur_fp {
                for item in delta_gain[0].iter_mut() {
                    *item += weight;
                }
            } else if part_u == cur_fp {
                for item in delta_gain[1].iter_mut() {
                    *item += weight;
                }
            } else {
                delta_gain[0][cur_fp as usize] -= weight;
                delta_gain[1][cur_fp as usize] -= weight;
                if part_w == cur_tp || part_u == cur_tp {
                    for k in 0..nparts {
                        self.delta_gain_v[k] -= weight;
                    }
                }
            }
            std::mem::swap(&mut cur_fp, &mut cur_tp);
        }

        delta_gain
    }

    pub fn update_move_general_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<Vec<i32>> {
        let (_, fp, tp) = (move_info.v, move_info.from_part, move_info.to_part);
        let nparts = self.num_parts as usize;
        let degree = self.idx_vec.len();

        let mut num = vec![0usize; nparts];
        for &w in &self.idx_vec {
            let p = part[self.hyprgraph.module_index(w)] as usize;
            if p < nparts {
                num[p] += 1;
            }
        }

        let mut delta_gain = vec![vec![0i32; nparts]; degree];
        let weight = self.hyprgraph.get_net_weight(move_info.net) as i32;

        let mut cur_fp = fp;
        let mut cur_tp = tp;
        let mut cur_weight = weight;
        for _ in 0..2 {
            let fp_idx = cur_fp as usize;
            if num[fp_idx] == 0 {
                for row in delta_gain.iter_mut() {
                    row[fp_idx] -= cur_weight;
                }
                if num[cur_tp as usize] > 0 {
                    for k in 0..nparts {
                        self.delta_gain_v[k] -= cur_weight;
                    }
                }
            } else if num[fp_idx] == 1 {
                let mut index = 0;
                while part[self.hyprgraph.module_index(self.idx_vec[index])] != cur_fp {
                    index += 1;
                }
                for item in delta_gain[index].iter_mut() {
                    *item += cur_weight;
                }
            }
            cur_weight = -cur_weight;
            std::mem::swap(&mut cur_fp, &mut cur_tp);
        }

        delta_gain
    }
}

use crate::fm_gain_mgr::GainCalcTrait;

impl<Gnl: Hypergraph> GainCalcTrait<Gnl> for FMKWayGainCalc<Gnl> {
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

    fn update_move_2pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) -> Gnl::Node {
        self.update_move_2pin_net(part, move_info)
    }

    fn update_move_3pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::Node>) -> Vec<i32> {
        // Convert Vec<Vec<i32>> to flat Vec<i32> for the trait interface
        let result = self.update_move_3pin_net(part, move_info);
        if result.is_empty() {
            Vec::new()
        } else {
            result[0].clone()
        }
    }

    fn update_move_general_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<i32> {
        let result = self.update_move_general_net(part, move_info);
        if result.is_empty() {
            Vec::new()
        } else {
            result[0].clone()
        }
    }

    fn delta_gain_w(&self) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    #[test]
    fn test_new_constructor() {
        let netlist = SimpleNetlist::new(4, 2);
        let calc = FMKWayGainCalc::new(netlist, 3);
        assert_eq!(calc.init_gain_matrix.len(), 3);
        assert_eq!(calc.init_gain_matrix[0].len(), 4);
        assert_eq!(calc.num_parts, 3);
        assert_eq!(calc.total_cost, 0);
    }

    #[test]
    fn test_init_no_nets() {
        let netlist = SimpleNetlist::new(4, 0);
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        let part = vec![0u8, 1, 2, 0];
        let cost = calc.init(&part);
        assert_eq!(cost, 0);
    }

    #[test]
    fn test_init_2pin_same_part() {
        let mut netlist = SimpleNetlist::new(4, 1);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[2], nodes[4]);
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        let part = vec![0u8, 0, 0, 1];
        let cost = calc.init(&part);
        // Both nodes in part 0: both get -weight in all other parts
        assert_eq!(cost, 0);
        assert_eq!(calc.init_gain_matrix[1][0], -1);
        assert_eq!(calc.init_gain_matrix[2][0], -1);
    }

    #[test]
    fn test_init_2pin_diff_part() {
        let mut netlist = SimpleNetlist::new(4, 1);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[2], nodes[4]);
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        let part = vec![0u8, 0, 1, 1];
        let cost = calc.init(&part);
        // Nodes 0 in part 0, node 2 in part 1: cross-part gain
        assert_eq!(cost, 1);
        // node 0 gets +weight in part 1
        assert_eq!(calc.init_gain_matrix[1][0], 1);
    }

    #[test]
    fn test_init_3pin_all_same() {
        let mut netlist = SimpleNetlist::new(3, 1);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[3]);
        netlist.add_edge(nodes[1], nodes[3]);
        netlist.add_edge(nodes[2], nodes[3]);
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        let part = vec![0u8, 0, 0];
        let cost = calc.init(&part);
        assert_eq!(cost, 0);
        // All in same part: all get -weight everywhere else
        assert_eq!(calc.init_gain_matrix[1][0], -1);
        assert_eq!(calc.init_gain_matrix[2][0], -1);
    }

    #[test]
    fn test_init_general_net_3_parts() {
        let mut netlist = SimpleNetlist::new(6, 1);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        for i in 0..6 {
            netlist.add_edge(nodes[i], nodes[6]);
        }
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        let part = vec![0u8, 0, 1, 1, 2, 2];
        let cost = calc.init(&part);
        // 3 partitions, each with 2 modules
        // total_cost = weight * (3 - 1) = 2
        assert_eq!(cost, 2);
        // Each partition has >1 module, so no one gets positive gain
        // Modules in partitions with count==0 get -weight
        // But every partition has 2 modules...
        // So no negative gain either
    }

    #[test]
    fn test_modify_gain_basic() {
        let netlist = SimpleNetlist::new(2, 0);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        let mut calc = FMKWayGainCalc::new(netlist, 3);
        calc.modify_gain(nodes[0], 0, 5);
        assert_eq!(calc.init_gain_matrix[1][0], 5);
        assert_eq!(calc.init_gain_matrix[2][0], 5);
        assert_eq!(calc.init_gain_matrix[0][0], 0);
    }
}
