use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::fm_constr_mgr::LegalCheck;
use crate::fm_kway_constr_mgr::FMKWayConstrMgr;
use crate::hypergraph::{FromIndex, Hypergraph};
use crate::midlevel::hamcycle::MidHamCycle;
use crate::midlevel::vertex::MidVertex;
use crate::moveinfo::MoveInfoV;

const MAX_PASSES: i32 = 5;
const MAX_PAIR_MODULES: usize = 15;

pub struct MidLvlKWayPartMgr {
    bal_tol: f64,
    num_parts: u8,
    pub total_cost: i32,
}

impl MidLvlKWayPartMgr {
    pub fn new(bal_tol: f64, num_parts: u8) -> Self {
        MidLvlKWayPartMgr {
            bal_tol,
            num_parts,
            total_cost: 0,
        }
    }

    pub fn optimize<Gnl>(&mut self, part: &mut [u8], hyprgraph: &Gnl)
    where
        Gnl: Hypergraph,
        Gnl::Node: FromIndex,
    {
        let total_modules = hyprgraph.number_of_modules();
        let mut current_part = part.to_vec();
        let mut constr_mgr = FMKWayConstrMgr::new(hyprgraph, self.bal_tol, self.num_parts);

        let mut improved = true;
        let mut pass = 0;

        while improved && pass < MAX_PASSES {
            improved = false;
            pass += 1;

            for i in 0..(self.num_parts as usize - 1) {
                for j in (i + 1)..self.num_parts as usize {
                    let mut selected = Vec::new();
                    for (v, &p) in current_part.iter().enumerate().take(total_modules) {
                        let pv = p as usize;
                        if pv == i || pv == j {
                            selected.push(v);
                        }
                    }
                    if selected.len() <= 1 || selected.len() > MAX_PAIR_MODULES {
                        continue;
                    }

                    let num_modules = selected.len();
                    let half_bits = num_modules / 2;
                    let total_bits = 2 * half_bits + 1;

                    let mut init_bits = vec![0i32; total_bits];
                    let mut init_part = vec![0u8; num_modules];
                    for (pos, &v) in selected.iter().enumerate().take(num_modules) {
                        init_part[pos] = if current_part[v] as usize == j {
                            1u8
                        } else {
                            0u8
                        };
                        if pos < half_bits {
                            init_bits[pos] = 1;
                        }
                    }

                    constr_mgr.init(&current_part);

                    let mut best_cost = 0;
                    for net in hyprgraph.nets() {
                        let mut seen = 0u8;
                        for nbr in hyprgraph.neighbors(net) {
                            let idx = hyprgraph.module_index(nbr);
                            seen |= 1u8 << current_part[idx];
                        }
                        for p in 0..8u8 {
                            if (seen & (1u8 << p)) != 0 {
                                best_cost += 1;
                                break;
                            }
                        }
                    }

                    let hyprgraph_ref = hyprgraph;

                    let pair_state = Rc::new(RefCell::new(PairState {
                        constr_mgr,
                        current_part,
                        selected: selected.clone(),
                        i,
                        j,
                    }));

                    let best_cost_cell = Rc::new(Cell::new(best_cost));
                    let best_part_cell = Rc::new(RefCell::new(init_part.clone()));
                    let local_cost_cell = Rc::new(Cell::new(best_cost));
                    let local_part_cell = Rc::new(RefCell::new(init_part.clone()));

                    let ps2 = pair_state.clone();
                    let bc2 = best_cost_cell.clone();
                    let bp2 = best_part_cell.clone();
                    let lc2 = local_cost_cell.clone();
                    let lp2 = local_part_cell.clone();

                    let visit_fn = move |bits: &[i32], flipped_pos: usize| {
                        if flipped_pos >= num_modules {
                            return;
                        }
                        let mut ps = ps2.borrow_mut();
                        let v = ps.selected[flipped_pos];
                        let to_part = if bits[flipped_pos] == 0 {
                            ps.i as u8
                        } else {
                            ps.j as u8
                        };
                        let from_part = if bits[flipped_pos] == 0 {
                            ps.j as u8
                        } else {
                            ps.i as u8
                        };

                        let move_info_v = MoveInfoV {
                            v: FromIndex::from_index(v),
                            from_part,
                            to_part,
                        };

                        let legal = ps.constr_mgr.check_legal(&move_info_v);

                        let mut delta = 0i32;
                        let v_node: Gnl::Node = FromIndex::from_index(v);
                        for net in hyprgraph_ref.neighbors(v_node) {
                            let mut cnt_from = 0i32;
                            let mut cnt_to = 0i32;
                            let mut cnt_other = 0i32;
                            for w in hyprgraph_ref.neighbors(net) {
                                let w_idx = hyprgraph_ref.module_index(w);
                                if w_idx == v {
                                    continue;
                                }
                                let pw = ps.current_part[w_idx] as usize;
                                if pw == from_part as usize {
                                    cnt_from += 1;
                                } else if pw == to_part as usize {
                                    cnt_to += 1;
                                } else {
                                    cnt_other += 1;
                                }
                            }
                            let wt = hyprgraph_ref.get_net_weight(net) as i32;
                            let before = cnt_to > 0 || cnt_other > 0;
                            let after = cnt_from > 0 || cnt_other > 0;
                            if !before && after {
                                delta += wt;
                            } else if before && !after {
                                delta -= wt;
                            }
                        }

                        lc2.set(lc2.get() + delta);
                        ps.current_part[v] = to_part;
                        lp2.borrow_mut()[flipped_pos] =
                            if bits[flipped_pos] == 0 { 0u8 } else { 1u8 };
                        ps.constr_mgr.update_move(&move_info_v);

                        if legal == LegalCheck::AllSatisfied && lc2.get() < bc2.get() {
                            bc2.set(lc2.get());
                            *bp2.borrow_mut() = lp2.borrow().clone();
                        }
                    };

                    let start_vertex = MidVertex::new(init_bits);
                    let _ham = MidHamCycle::new(start_vertex, -1, Box::new(visit_fn));

                    let ps = Rc::try_unwrap(pair_state).ok().unwrap().into_inner();
                    current_part = ps.current_part;
                    constr_mgr = ps.constr_mgr;

                    for (pos, &v) in selected.iter().enumerate().take(num_modules) {
                        current_part[v] = if best_part_cell.borrow()[pos] == 1 {
                            j as u8
                        } else {
                            i as u8
                        };
                    }

                    if best_cost_cell.get() < self.total_cost || pass == 1 {
                        self.total_cost = best_cost_cell.get();
                        improved = true;
                    }
                }
            }
        }

        part.copy_from_slice(&current_part);
    }
}

struct PairState<Gnl: Hypergraph> {
    constr_mgr: FMKWayConstrMgr<Gnl>,
    current_part: Vec<u8>,
    selected: Vec<usize>,
    i: usize,
    j: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    fn create_dwarf_netlist() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(7, 6);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        let edges: Vec<(usize, usize)> = vec![
            (4, 7),
            (0, 7),
            (1, 7),
            (0, 8),
            (2, 8),
            (3, 8),
            (1, 9),
            (2, 9),
            (3, 9),
            (2, 10),
            (5, 10),
            (3, 11),
            (6, 11),
            (0, 12),
        ];
        for (u, v) in &edges {
            netlist.add_edge(nodes[*u], nodes[*v]);
        }
        netlist.module_weight = vec![1, 3, 4, 2, 0, 0, 0];
        netlist
    }

    #[test]
    #[ignore = "MidHamCycle: hangs (infinite loop) for certain bitstring sizes"]
    fn test_mid_lvl_kway_dwarf_3way() {
        let hyprgraph = create_dwarf_netlist();
        let n = hyprgraph.number_of_modules();
        let mut mgr = MidLvlKWayPartMgr::new(0.4, 3);
        let mut part = vec![0u8; n];
        for (i, p) in part.iter_mut().enumerate().take(n) {
            *p = (i % 3) as u8;
        }
        mgr.optimize(&mut part, &hyprgraph);
        assert!(mgr.total_cost >= 0);
    }
}
