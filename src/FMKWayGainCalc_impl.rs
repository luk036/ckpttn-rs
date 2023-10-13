#include <assert.h>  // for assert
#include <stdint.h>  // for u8

// #include <__config>                                           // for std
// #include <__hash_table>                                       // for __hash_...
#include <algorithm>                  // for fill
#include <ckpttn/FMKWayGainCalc.hpp>  // for FMKWayG...
#include <ckpttn/FMPmrConfig.hpp>     // for FM_MAX_...
#include <cstddef>                    // for byte
#include <gsl/span>                   // for span
#include <initializer_list>           // for initial...
#include <type_traits>                // for swap
#include <utility>                    // for pair
#include <Vec>                     // for Vec

#include "ckpttn/dllist.hpp"    // for Dllink
#include "ckpttn/moveinfo.hpp"  // for MoveInfo
#include "ckpttn/robin.hpp"     // for Robin<>...

// using namespace ranges;
using namespace std;

impl<Gnl> FMKWayGainCalc<Gnl> {

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     * @param[in] vertex_list
     */
    fn init_gain(&mut self, net: &Gnl::node_t, part: &[u8]) {
        let degree = self.hyprgraph.gr.degree(net);
        if degree < 2 || degree > FM_MAX_DEGREE  // [[unlikely]]
        {
            return;  // does not provide any gain when moving
        }
        if !special_handle_2pin_nets {
            self._init_gain_general_net(net, part);
            return;
        }
        switch (degree) {
            case 2:
                self._init_gain_2pin_net(net, part);
                break;
            case 3:
                self._init_gain_3pin_net(net, part);
                break;
            default:
                self._init_gain_general_net(net, part);
        }
    }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_2pin_net(&mut self, net: &Gnl::node_t, part: &[u8]) {
        let mut net_cur = self.hyprgraph.gr[net].begin();
        let w = *net_cur;
        let v = *++net_cur;
        let part_w = part[w];
        let part_v = part[v];
        let weight = self.hyprgraph.get_net_weight(net);
        if part_v == part_w {
            self._modify_gain(w, part_v, -weight);
            self._modify_gain(v, part_v, -weight);
            // self._modify_gain_va(-weight, part_v, w, v);
        } else {
            self.totalcost += weight;
            self.vertex_list[part_v][w].data.second += weight;
            self.vertex_list[part_w][v].data.second += weight;
        }
    }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_3pin_net(&mut self, net: &Gnl::node_t, part: &[u8]) {
        let mut net_cur = self.hyprgraph.gr[net].begin();
        let w = *net_cur;
        let v = *++net_cur;
        let u = *++net_cur;
        let part_w = part[w];
        let part_v = part[v];
        let part_u = part[u];
        let weight = self.hyprgraph.get_net_weight(net);
        let mut a = w;
        let mut b = v;
        let mut c = u;

        if part_u == part_v {
            if part_w == part_v {
                self._modify_gain(u, part_v, -weight);
                self._modify_gain(v, part_v, -weight);
                self._modify_gain(w, part_v, -weight);
                // self._modify_gain_va(-weight, part_v, u, v, w);
                return;
            }
        } else if part_w == part_v {
            a = u, b = w, c = v;
        } else if part_w == part_u {
            a = v, b = u, c = w;
        } else {
            self.totalcost += 2 * weight;
            // self._modify_vertex_va(weight, part_v, u, w);
            // self._modify_vertex_va(weight, part_w, u, v);
            // self._modify_vertex_va(weight, part_u, v, w);
            self._modify_gain(u, part_v, weight);
            self._modify_gain(w, part_v, weight);
            self._modify_gain(u, part_w, weight);
            self._modify_gain(v, part_w, weight);
            self._modify_gain(v, part_u, weight);
            self._modify_gain(w, part_u, weight);
            return;
        }

        for e in {b, c}.iter() {
            self._modify_gain(e, part[b], -weight);
            self.vertex_list[part[a]][e].data.second += weight;
        }
        self.vertex_list[part[b]][a].data.second += weight;

        // self._modify_gain_va(-weight, part[b], b, c);
        // self._modify_vertex_va(weight, part[a], b, c);
        // self._modify_vertex_va(weight, part[b], a);

        self.totalcost += weight;
    }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_general_net(&mut self, net: &Gnl::node_t, part: &[u8]) {
        let mut num = FMPmr::Vec<u8>(self.num_parts, 0);
        // let mut idx_vec = FMPmr::Vec<Gnl::node_t>(&rsrc);

        for w in self.hyprgraph.gr[net].iter() {
            num[part[w]] += 1;
            // idx_vec.push(w);
        }
        let weight = self.hyprgraph.get_net_weight(net);
        for c in num.iter() {
            if c > 0 {
                self.totalcost += weight;
            }
        }
        self.totalcost -= weight;

        // for (let & [k, c] : views::enumerate(num))
        let mut k = 0U;
        for c in num.iter() {
            if c == 0 {
                for w in self.hyprgraph.gr[net].iter() {
                    vertex_list[k][w].data.second -= weight;
                }
            } else if c == 1 {
                for w in self.hyprgraph.gr[net].iter() {
                    if part[w] == k {
                        self._modify_gain(w, part[w], weight);
                        break;
                    }
                }
            }
            ++k;
        }
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @param[out] w
     * @return ret_2pin_info
     */
    pub fn update_move_2pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::node_t>) -> Gnl::node_t {
        // let & [net, v, from_part, to_part] = move_info;
        assert!(part[move_info.v] == move_info.from_part);

        let mut weight = self.hyprgraph.get_net_weight(move_info.net);
        // let mut delta_gain_w = Vec<i32>(self.num_parts, 0);
        let mut net_cur = self.hyprgraph.gr[move_info.net].begin();
        let mut w = (*net_cur != move_info.v) ? *net_cur : *++net_cur;
        fill(self.delta_gain_w.begin(), self.delta_gain_w.end(), 0);

        // #pragma unroll
        for l in {move_info.from_part, move_info.to_part}.iter() {
            if part[w] == l {
                // for (let mut i = 0U; i != delta_gain_w.len(); ++i)
                // {
                //     delta_gain_w[i] += weight;
                //     delta_gain_v[i] += weight;
                // }
                for dgw in delta_gain_w.iter_mut() {
                    dgw += weight;
                }
                for dgv in delta_gain_v.iter_mut() {
                    dgv += weight;
                }
            }
            delta_gain_w[l] -= weight;
            weight = -weight;
        }
        return w;
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    pub fn init_idx_vec(&mut self, v: &Gnl::node_t, net: &Gnl::node_t) {
        self.idx_vec.clear();
        for w in self.hyprgraph.gr[net].iter() {
            if w == v {
                continue;
            }
            self.idx_vec.push(w);
        }
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    pub fn update_move_3pin_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::node_t>)
        -> FMKWayGainCalc<Gnl>::ret_info {
        let degree = self.idx_vec.len();
        let mut delta_gain = vec![vec![0i32; self.num_parts]; degree];
        let mut weight = self.hyprgraph.get_net_weight(move_info.net);
        let part_w = part[self.idx_vec[0]];
        let part_u = part[self.idx_vec[1]];
        let mut l = move_info.from_part;
        let mut u = move_info.to_part;

        if part_w == part_u {
            // #pragma unroll
            for (let mut i = 0; i != 2; ++i) {
                if part_w != l {
                    delta_gain[0][l] -= weight;
                    delta_gain[1][l] -= weight;
                    if part_w == u {
                        for dgv in self.delta_gain_v.iter_mut() {
                            *dgv -= weight;
                        }
                    }
                }
                weight = -weight;
                let temp = l;
                l = u;
                u = temp;    
            }
            return delta_gain;
        }

        // #pragma unroll
        for (let mut i = 0; i != 2; ++i) {
            if part_w == l {
                for dg0 in delta_gain[0].iter_mut() {
                    *dg0 += weight;
                }
            } else if part_u == l {
                for dg1 in delta_gain[1].iter_mut() {
                    *dg1 += weight;
                }
            } else {
                delta_gain[0][l] -= weight;
                delta_gain[1][l] -= weight;
                if part_w == u || part_u == u {
                    for dgv in self.delta_gain_v.iter_mut() {
                        *dgv -= weight;
                    }
                }
            }
            weight = -weight;
            let temp = l;
            l = u;
            u = temp;
        }
        return delta_gain;
        // return self.update_move_general_net(part, move_info);
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    pub fn update_move_general_net(&mut self, part: &[u8], move_info: &MoveInfo<Gnl::node_t>)
        -> FMKWayGainCalc<Gnl>::ret_info {
        let mut num = vec![0u8; self.num_parts];
        for w in self.idx_vec.iter() {
            num[part[w]] += 1;
        }
        let degree = self.idx_vec.len();
        let mut delta_gain = vec![vec![0i32; self.num_parts]; degree];
        let mut weight = self.hyprgraph.get_net_weight(move_info.net);

        let mut l = move_info.from_part;
        let mut u = move_info.to_part;

        // #pragma unroll
        for i in 0..2 {
            if num[l] == 0 {
                for index in 0..degree {
                    delta_gain[index][l] -= weight;
                }
                if num[u] > 0 {
                    for dgv in self.delta_gain_v.iter_mut() {
                        *dgv -= weight;
                    }
                }
            } else if num[l] == 1 {
                for index in 0..degree {
                    if part[self.idx_vec[index]] == l {
                        for dg in delta_gain[index].iter_mut() {
                            *dg += weight;
                        }
                        break;
                    }
                }
            }
            weight = -weight;
            let temp = l;
            l = u;
            u = temp;
        };
        return delta_gain;
    }
}

// // instantiation

// #include <py2cpp/set.hpp>              // for set
// #include <xnetwork/classes/graph.hpp>  // for Graph

// #include "ckpttn/netlist.hpp"  // for Netlist

// template class FMKWayGainCalc<SimpleNetlist>;