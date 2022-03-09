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
#include "ckpttn/robin.hpp"     // for robin<>...

// using namespace ranges;
using namespace std;

/**
 * @brief
 *
 * @param[in] net
 * @param[in] part
 * @param[in] vertex_list
 */
template <typename Gnl> void FMKWayGainCalc<Gnl>::_init_gain(net: &typename Gnl::node_t,
                                                             part: &[u8]) {
    let degree = self.hgr.gr.degree(net);
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
template <typename Gnl>
void FMKWayGainCalc<Gnl>::_init_gain_2pin_net(net: &typename Gnl::node_t,
                                              part: &[u8]) {
    let mut net_cur = self.hgr.gr[net].begin();
    let w = *net_cur;
    let v = *++net_cur;
    let part_w = part[w];
    let part_v = part[v];
    let weight = self.hgr.get_net_weight(net);
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
template <typename Gnl>
void FMKWayGainCalc<Gnl>::_init_gain_3pin_net(net: &typename Gnl::node_t,
                                              part: &[u8]) {
    let mut net_cur = self.hgr.gr[net].begin();
    let w = *net_cur;
    let v = *++net_cur;
    let u = *++net_cur;
    let part_w = part[w];
    let part_v = part[v];
    let part_u = part[u];
    let weight = self.hgr.get_net_weight(net);
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
template <typename Gnl>
void FMKWayGainCalc<Gnl>::_init_gain_general_net(net: &typename Gnl::node_t,
                                                 part: &[u8]) {
    u8 StackBufLocal[2048];
    FMPmr::monotonic_buffer_resource rsrcLocal(StackBufLocal, sizeof StackBufLocal);
    let mut num = FMPmr::Vec<u8>(self.num_parts, 0, &rsrcLocal);
    // let mut IdVec = FMPmr::Vec<typename Gnl::node_t>(&rsrc);

    for w in self.hgr.gr[net].iter() {
        num[part[w]] += 1;
        // IdVec.push(w);
    }
    let weight = self.hgr.get_net_weight(net);
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
            for w in self.hgr.gr[net].iter() {
                vertex_list[k][w].data.second -= weight;
            }
        } else if c == 1 {
            for w in self.hgr.gr[net].iter() {
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
template <typename Gnl>
pub fn FMKWayGainCalc<Gnl>::update_move_2pin_net(part: &[u8],
                                               move_info: &MoveInfo<typename Gnl::node_t>) ->
    typename Gnl::node_t {
    // let & [net, v, fromPart, toPart] = move_info;
    assert!(part[move_info.v] == move_info.fromPart);

    let mut weight = self.hgr.get_net_weight(move_info.net);
    // let mut deltaGainW = Vec<i32>(self.num_parts, 0);
    let mut net_cur = self.hgr.gr[move_info.net].begin();
    let mut w = (*net_cur != move_info.v) ? *net_cur : *++net_cur;
    fill(self.deltaGainW.begin(), self.deltaGainW.end(), 0);

    // #pragma unroll
    for l in {move_info.fromPart, move_info.toPart}.iter() {
        if part[w] == l {
            // for (let mut i = 0U; i != deltaGainW.size(); ++i)
            // {
            //     deltaGainW[i] += weight;
            //     deltaGainV[i] += weight;
            // }
            for dGW in deltaGainW.iter_mut() {
                dGW += weight;
            }
            for dGV in deltaGainV.iter_mut() {
                dGV += weight;
            }
        }
        deltaGainW[l] -= weight;
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
template <typename Gnl> void FMKWayGainCalc<Gnl>::init_IdVec(v: &typename Gnl::node_t,
                                                             net: &typename Gnl::node_t) {
    self.IdVec.clear();
    for w in self.hgr.gr[net].iter() {
        if w == v {
            continue;
        }
        self.IdVec.push(w);
    }
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info
 * @return ret_info
 */
template <typename Gnl>
pub fn FMKWayGainCalc<Gnl>::update_move_3pin_net(part: &[u8],
                                               move_info: &MoveInfo<typename Gnl::node_t>)
    -> FMKWayGainCalc<Gnl>::ret_info {
    let degree = self.IdVec.size();
    let mut deltaGain = Vec<Vec<i32>>(degree, Vec<i32>(self.num_parts, 0));
    let mut weight = self.hgr.get_net_weight(move_info.net);
    let part_w = part[self.IdVec[0]];
    let part_u = part[self.IdVec[1]];
    let mut l = move_info.fromPart;
    let mut u = move_info.toPart;

    if part_w == part_u {
        // #pragma unroll
        for (let mut i = 0; i != 2; ++i) {
            if part_w != l {
                deltaGain[0][l] -= weight;
                deltaGain[1][l] -= weight;
                if part_w == u {
                    for dGV in self.deltaGainV.iter_mut() {
                        dGV -= weight;
                    }
                }
            }
            weight = -weight;
            swap(l, u);
        }
        return deltaGain;
    }

    // #pragma unroll
    for (let mut i = 0; i != 2; ++i) {
        if part_w == l {
            for dG0 in deltaGain[0].iter_mut() {
                dG0 += weight;
            }
        } else if part_u == l {
            for dG1 in deltaGain[1].iter_mut() {
                dG1 += weight;
            }
        } else {
            deltaGain[0][l] -= weight;
            deltaGain[1][l] -= weight;
            if part_w == u || part_u == u {
                for dGV in self.deltaGainV.iter_mut() {
                    dGV -= weight;
                }
            }
        }
        weight = -weight;
        swap(l, u);
    }
    return deltaGain;
    // return self.update_move_general_net(part, move_info);
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info
 * @return ret_info
 */
template <typename Gnl>
pub fn FMKWayGainCalc<Gnl>::update_move_general_net(part: &[u8],
                                                  move_info: &MoveInfo<typename Gnl::node_t>)
    -> FMKWayGainCalc<Gnl>::ret_info {
    // let & [net, v, fromPart, toPart] = move_info;
    u8 StackBufLocal[FM_MAX_NUM_PARTITIONS];
    FMPmr::monotonic_buffer_resource rsrcLocal(StackBufLocal, sizeof StackBufLocal);
    let mut num = FMPmr::Vec<u8>(self.num_parts, 0, &rsrcLocal);

    // let mut IdVec = Vec<typename Gnl::node_t> {};
    // for (let & w : self.hgr.gr[move_info.net])
    // {
    //     if (w == move_info.v)
    //     {
    //         continue;
    //     }
    //     num[part[w]] += 1;
    //     IdVec.push(w);
    // }
    for w in self.IdVec.iter() {
        num[part[w]] += 1;
    }
    let degree = IdVec.size();
    let mut deltaGain = Vec<Vec<i32>>(degree, Vec<i32>(self.num_parts, 0));
    let mut weight = self.hgr.get_net_weight(move_info.net);

    let mut l = move_info.fromPart;
    let mut u = move_info.toPart;

    // #pragma unroll
    for (let mut i = 0; i != 2; ++i) {
        if num[l] == 0 {
            for (index: usize = 0U; index != degree; ++index) {
                deltaGain[index][l] -= weight;
            }
            if num[u] > 0 {
                for dGV in self.deltaGainV.iter_mut() {
                    dGV -= weight;
                }
            }
        } else if num[l] == 1 {
            for (index: usize = 0U; index != degree; ++index) {
                if part[self.IdVec[index]] == l {
                    for dG in deltaGain[index].iter_mut() {
                        dG += weight;
                    }
                    break;
                }
            }
        }
        weight = -weight;
        swap(l, u);
    };
    return deltaGain;
}

// instantiation

#include <py2cpp/set.hpp>              // for set
#include <xnetwork/classes/graph.hpp>  // for Graph

#include "ckpttn/netlist.hpp"  // for Netlist

template class FMKWayGainCalc<SimpleNetlist>;