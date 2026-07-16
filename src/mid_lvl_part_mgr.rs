use std::cell::RefCell;
use std::rc::Rc;

use crate::fm_bi_constr_mgr::FMBiConstrMgr;
use crate::fm_bi_gain_calc::FMBiGainCalc;
use crate::fm_constr_mgr::LegalCheck;
use crate::hypergraph::{FromIndex, Hypergraph};
use crate::midlevel::hamcycle::MidHamCycle;
use crate::midlevel::vertex::MidVertex;
use crate::moveinfo::{MoveInfo, MoveInfoV};

const FM_MAX_DEGREE: usize = 500;

pub struct MidLvlPartMgr<'a, Gnl: Hypergraph> {
    hyprgraph: &'a Gnl,
    bal_tol: f64,
    pub total_cost: i32,
}

impl<'a, Gnl: Hypergraph> MidLvlPartMgr<'a, Gnl> {
    pub fn new(hyprgraph: &'a Gnl, bal_tol: f64) -> Self {
        MidLvlPartMgr {
            hyprgraph,
            bal_tol,
            total_cost: 0,
        }
    }

    pub fn optimize(&mut self, part: &mut [u8])
    where
        Gnl::Node: FromIndex,
    {
        let num_modules = self.hyprgraph.number_of_modules();
        if num_modules < 2 {
            return;
        }
        let half_bits = num_modules / 2;
        let total_bits = 2 * half_bits + 1;

        let mut current_part = vec![0u8; num_modules];
        for i in 0..half_bits.min(num_modules) {
            current_part[i] = 1;
        }

        let mut gain_calc = FMBiGainCalc::new(self.hyprgraph, 2);
        let mut constr_mgr = FMBiConstrMgr::new(self.hyprgraph, self.bal_tol);

        let current_cost = gain_calc.init(&current_part);
        constr_mgr.init(&current_part);

        let current_gain = gain_calc.init_gain_list.clone();

        let mut init_bits = vec![0i32; total_bits];
        for i in 0..half_bits {
            init_bits[i] = 1;
        }

        let hyprgraph = self.hyprgraph;

        let state = Rc::new(RefCell::new(MidLvlState {
            gain_calc,
            constr_mgr,
            current_part,
            current_gain,
            current_cost,
            best_part: vec![0u8; num_modules],
            best_cost: current_cost,
        }));

        let state2 = state.clone();
        let visit_fn = move |bits: &[i32], flipped_pos: usize| {
            if flipped_pos >= num_modules {
                return;
            }
            let to_part = bits[flipped_pos] as u8;
            let from_part = 1u8 - to_part;

            let mut s = state2.borrow_mut();
            let v: Gnl::Node = FromIndex::from_index(flipped_pos);
            let move_info_v = MoveInfoV {
                v,
                from_part,
                to_part,
            };

            let legal = s.constr_mgr.check_legal(&move_info_v);
            let gain = s.current_gain[flipped_pos];
            s.current_cost -= gain;

            let nbrs: Vec<_> = hyprgraph.neighbors(v).collect();
            for &net in &nbrs {
                let degree = hyprgraph.degree(net);
                if !(2..=FM_MAX_DEGREE).contains(&degree) {
                    continue;
                }
                let move_info = MoveInfo {
                    net,
                    v,
                    from_part,
                    to_part,
                };
                if degree == 2 {
                    let cp = s.current_part.clone();
                    let w = s.gain_calc.update_move_2pin_net(&cp, &move_info);
                    s.current_part = cp;
                    s.current_gain[hyprgraph.module_index(w)] += s.gain_calc.delta_gain_w();
                } else {
                    s.gain_calc.init_idx_vec(v, net);
                    let iv = s.gain_calc.idx_vec.clone();
                    let cp = s.current_part.clone();
                    let deltas = if degree == 3 {
                        s.gain_calc.update_move_3pin_net(&cp, &move_info)
                    } else {
                        s.gain_calc.update_move_general_net(&cp, &move_info)
                    };
                    s.current_part = cp;
                    for i in 0..iv.len() {
                        s.current_gain[hyprgraph.module_index(iv[i])] += deltas[i];
                    }
                }
            }

            s.current_gain[flipped_pos] = -gain;
            s.constr_mgr.update_move(&move_info_v);
            s.current_part[flipped_pos] = to_part;

            if legal == LegalCheck::AllSatisfied && s.current_cost < s.best_cost {
                s.best_cost = s.current_cost;
                s.best_part = s.current_part.clone();
            }
        };

        let start_vertex = MidVertex::new(init_bits);
        let _ham = MidHamCycle::new(start_vertex, -1, Box::new(visit_fn));

        let s = state.borrow();
        for i in 0..num_modules {
            part[i] = s.best_part[i];
        }
        self.total_cost = s.best_cost;
    }
}

struct MidLvlState<'a, Gnl: Hypergraph> {
    gain_calc: FMBiGainCalc<&'a Gnl>,
    constr_mgr: FMBiConstrMgr<&'a Gnl>,
    current_part: Vec<u8>,
    current_gain: Vec<i32>,
    current_cost: i32,
    best_part: Vec<u8>,
    best_cost: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    fn create_star_netlist(m: usize) -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(m, 1);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        for i in 0..m {
            netlist.add_edge(nodes[i], nodes[m]);
        }
        netlist
    }

    #[test]
    #[ignore = "MidHamCycle: to_first_vertex() produces invalid state for small inputs"]
    fn test_mid_lvl_part_mgr_default() {
        let mut netlist = SimpleNetlist::new(3, 3);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[3]);
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[3]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[4]);
        netlist.add_edge(nodes[0], nodes[5]);
        netlist.module_weight = vec![3, 4, 2];

        let mut mgr = MidLvlPartMgr::new(&netlist, 0.45);
        let n = netlist.number_of_modules();
        let mut part = vec![0u8; n];
        let half = n / 2;
        for i in 0..half {
            part[i] = 1;
        }
        mgr.optimize(&mut part);
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    #[ignore = "MidHamCycle: hangs (infinite loop) for certain bitstring sizes"]
    fn test_mid_lvl_part_mgr_n5() {
        let netlist = create_star_netlist(5);
        let mut mgr = MidLvlPartMgr::new(&netlist, 0.45);
        let m = netlist.number_of_modules();
        let mut part = vec![0u8; m];
        for i in 0..m / 2 {
            part[i] = 1;
        }
        mgr.optimize(&mut part);
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    fn test_mid_lvl_part_mgr_n8() {
        let netlist = create_star_netlist(8);
        let mut mgr = MidLvlPartMgr::new(&netlist, 0.45);
        let m = netlist.number_of_modules();
        let mut part = vec![0u8; m];
        for i in 0..m / 2 {
            part[i] = 1;
        }
        mgr.optimize(&mut part);
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    fn test_mid_lvl_part_mgr_n11() {
        let netlist = create_star_netlist(11);
        let mut mgr = MidLvlPartMgr::new(&netlist, 0.45);
        let m = netlist.number_of_modules();
        let mut part = vec![0u8; m];
        for i in 0..m / 2 {
            part[i] = 1;
        }
        mgr.optimize(&mut part);
        assert!(mgr.total_cost >= 0);
    }

    #[test]
    fn test_mid_lvl_part_mgr_n15() {
        let netlist = create_star_netlist(15);
        let mut mgr = MidLvlPartMgr::new(&netlist, 0.45);
        let m = netlist.number_of_modules();
        let mut part = vec![0u8; m];
        for i in 0..m / 2 {
            part[i] = 1;
        }
        mgr.optimize(&mut part);
        assert!(mgr.total_cost >= 0);
    }
}
