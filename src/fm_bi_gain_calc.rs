use crate::hypergraph::Hypergraph;
use crate::moveinfo::MoveInfo;

pub struct FMBiGainCalc<Gnl: Hypergraph> {
    hyprgraph: Gnl,
    pub init_gain_list: Vec<i32>,
    pub total_cost: i32,
    pub delta_gain_w_val: i32,
    pub idx_vec: Vec<Gnl::Node>,
    pub special_handle_2pin_nets: bool,
}

impl<Gnl: Hypergraph> FMBiGainCalc<Gnl> {
    pub fn new(hyprgraph: Gnl, _num_parts: u8) -> Self {
        let nmod = hyprgraph.number_of_modules();
        FMBiGainCalc {
            hyprgraph,
            init_gain_list: vec![0; nmod],
            total_cost: 0,
            delta_gain_w_val: 0,
            idx_vec: Vec::new(),
            special_handle_2pin_nets: true,
        }
    }

    pub fn init(&mut self, part: &[u8]) -> i32 {
        self.total_cost = 0;
        for elem in &mut self.init_gain_list {
            *elem = 0;
        }
        let nets: Vec<_> = self.hyprgraph.nets().collect();
        for net in nets {
            self.init_gain(net, part);
        }
        self.total_cost
    }

    pub fn update_move_init(&mut self) {}

    pub fn init_idx_vec(&mut self, v: Gnl::Node, net: Gnl::Node) {
        self.idx_vec.clear();
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let degree = nbrs.len();
        self.idx_vec.reserve(degree - 1);
        for w in nbrs {
            if w != v {
                self.idx_vec.push(w);
            }
        }
    }

    pub fn idx_vec(&self) -> &Vec<Gnl::Node> {
        &self.idx_vec
    }

    pub fn delta_gain_w(&self) -> i32 {
        self.delta_gain_w_val
    }

    pub fn update_move_2pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Gnl::Node {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(move_info.net).collect();
        let first = nbrs[0];
        let second = nbrs[1];
        let node_w = if first != move_info.v { first } else { second };
        let gain = self.hyprgraph.get_net_weight(move_info.net) as i32;
        let delta = if part[self.hyprgraph.module_index(node_w)] == move_info.from_part {
            gain
        } else {
            -gain
        };
        self.delta_gain_w_val = 2 * delta;
        node_w
    }

    pub fn update_move_3pin_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<i32> {
        let mut delta_gain = vec![0i32; 2];
        let gain = self.hyprgraph.get_net_weight(move_info.net) as i32;
        let part_w = part[self.hyprgraph.module_index(self.idx_vec[0])];
        let adjusted_gain = if part_w != move_info.from_part {
            -gain
        } else {
            gain
        };
        if part_w == part[self.hyprgraph.module_index(self.idx_vec[1])] {
            delta_gain[0] += adjusted_gain;
            delta_gain[1] += adjusted_gain;
        } else {
            delta_gain[0] += adjusted_gain;
            delta_gain[1] -= adjusted_gain;
        }
        delta_gain
    }

    pub fn update_move_general_net(
        &mut self,
        part: &[u8],
        move_info: &MoveInfo<Gnl::Node>,
    ) -> Vec<i32> {
        let degree = self.idx_vec.len();
        let mut delta_gain = vec![0i32; degree];
        let gain = self.hyprgraph.get_net_weight(move_info.net) as i32;

        let mut num = [0usize; 2];
        for &w in &self.idx_vec {
            let p = part[self.hyprgraph.module_index(w)] as usize;
            if p < 2 {
                num[p] += 1;
            }
        }

        let mut current_gain = gain;
        for &target_part in &[move_info.from_part, move_info.to_part] {
            let tp = target_part as usize;
            if num[tp] == 0 {
                for d in &mut delta_gain {
                    *d -= current_gain;
                }
            } else if num[tp] == 1 {
                for (i, &w) in self.idx_vec.iter().enumerate() {
                    if part[self.hyprgraph.module_index(w)] == target_part {
                        delta_gain[i] += current_gain;
                        break;
                    }
                }
            }
            current_gain = -current_gain;
        }
        delta_gain
    }

    fn init_gain(&mut self, net: Gnl::Node, part: &[u8]) {
        let degree = self.hyprgraph.degree(net);
        if !(2..=65536).contains(&degree) {
            return;
        }
        if !self.special_handle_2pin_nets {
            self.init_gain_general_net(net, part);
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
        let node_w = nbrs[0];
        let node_v = nbrs[1];
        let weight = self.hyprgraph.get_net_weight(net) as i32;
        if part[self.hyprgraph.module_index(node_w)] != part[self.hyprgraph.module_index(node_v)] {
            self.total_cost += weight;
            self.increase_gain(node_w, weight as u32);
            self.increase_gain(node_v, weight as u32);
        } else {
            self.decrease_gain(node_w, weight as u32);
            self.decrease_gain(node_v, weight as u32);
        }
    }

    fn init_gain_3pin_net(&mut self, net: Gnl::Node, part: &[u8]) {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let node_w = nbrs[0];
        let node_v = nbrs[1];
        let node_u = nbrs[2];
        let weight = self.hyprgraph.get_net_weight(net) as i32;

        let pu = part[self.hyprgraph.module_index(node_u)];
        let pv = part[self.hyprgraph.module_index(node_v)];
        let pw = part[self.hyprgraph.module_index(node_w)];

        if pu == pv {
            if pw == pv {
                self.decrease_gain(node_u, weight as u32);
                self.decrease_gain(node_v, weight as u32);
                self.decrease_gain(node_w, weight as u32);
                return;
            }
            self.increase_gain(node_w, weight as u32);
        } else if pw == pv {
            self.increase_gain(node_u, weight as u32);
        } else {
            self.increase_gain(node_v, weight as u32);
        }
        self.total_cost += weight;
    }

    fn init_gain_general_net(&mut self, net: Gnl::Node, part: &[u8]) {
        let nbrs: Vec<_> = self.hyprgraph.neighbors(net).collect();
        let mut num = [0usize; 2];
        for &w in &nbrs {
            let p = part[self.hyprgraph.module_index(w)] as usize;
            if p < 2 {
                num[p] += 1;
            }
        }
        let weight = self.hyprgraph.get_net_weight(net) as i32;

        for (part_idx, &n) in num.iter().enumerate() {
            if n == 0 {
                for &w in &nbrs {
                    self.decrease_gain(w, weight as u32);
                }
            } else if n == 1 {
                for &w in &nbrs {
                    if part[self.hyprgraph.module_index(w)] as usize == part_idx {
                        self.increase_gain(w, weight as u32);
                        break;
                    }
                }
            }
        }
        if num[0] > 0 && num[1] > 0 {
            self.total_cost += weight;
        }
    }

    fn increase_gain(&mut self, w: Gnl::Node, weight: u32) {
        let idx = self.hyprgraph.module_index(w);
        if idx < self.init_gain_list.len() {
            self.init_gain_list[idx] += weight as i32;
        }
    }

    fn decrease_gain(&mut self, w: Gnl::Node, weight: u32) {
        let idx = self.hyprgraph.module_index(w);
        if idx < self.init_gain_list.len() {
            self.init_gain_list[idx] -= weight as i32;
        }
    }
}
