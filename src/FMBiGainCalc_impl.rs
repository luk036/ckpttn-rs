// #include <__config>                    // for std
// #include <__hash_table>                // for __hash_const_iterator, operator!=
// #include <array>                           // for array
// #include <boost/container/pmr/Vec.hpp>  // for Vec
// #include <boost/container/Vec.hpp>      // for operator!=, vec_iterator
// #include <ckpttn/FMBiGainCalc.hpp>         // for FMBiGainCalc, part, net
// #include <ckpttn/FMPmrConfig.hpp>          // for FM_MAX_DEGREE
// #include <cstddef>                         // for usize
// #include <cstdint>                         // for u8
// #include <gsl/span>                        // for span
// #include <initializer_list>                // for initializer_list
// #include <Vec>                          // for Vec

// #include "ckpttn/moveinfo.hpp"  // for MoveInfo

// #include <range/v3/view/remove_if.hpp>
// #include <transrangers.hpp>
// #include <range/v3/view/all.hpp>

// using namespace std;

/**
 * @brief
 *
 * @param[in] net
 * @param[in] part
 */
template <Gnl>
void FMBiGainCalc<Gnl>::_init_gain(net: &Gnl::node_t, part: &[u8]) {
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
template <Gnl> void FMBiGainCalc<Gnl>::_init_gain_2pin_net(net: &Gnl::node_t,
                                                                    part: &[u8]) {
    let mut net_cur = self.hyprgraph.gr[net].begin();
    let w = *net_cur;
    let v = *++net_cur;

    let weight = self.hyprgraph.get_net_weight(net);
    if part[w] != part[v] {
        self.totalcost += weight;
        // self._modify_gain_va(weight, w, v);
        self._modify_gain(w, weight);
        self._modify_gain(v, weight);
    } else {
        // self._modify_gain_va(-weight, w, v);
        self._modify_gain(w, -weight);
        self._modify_gain(v, -weight);
    }
}

/**
 * @brief
 *
 * @param[in] net
 * @param[in] part
 */
template <Gnl> void FMBiGainCalc<Gnl>::_init_gain_3pin_net(net: &Gnl::node_t,
                                                                    part: &[u8]) {
    let mut net_cur = self.hyprgraph.gr[net].begin();
    let w = *net_cur;
    let v = *++net_cur;
    let u = *++net_cur;

    let weight = self.hyprgraph.get_net_weight(net);
    if part[u] == part[v] {
        if part[w] == part[v] {
            // self._modify_gain_va(-weight, u, v, w);
            self._modify_gain(u, -weight);
            self._modify_gain(v, -weight);
            self._modify_gain(w, -weight);
            return;
        }
        // self._modify_gain_va(weight, w);
        self._modify_gain(w, weight);
    } else if part[w] == part[v] {
        self._modify_gain(u, weight);
    } else {
        self._modify_gain(v, weight);
    }
    self.totalcost += weight;
}

/**
 * @brief
 *
 * @param[in] net
 * @param[in] part
 */
template <Gnl>
void FMBiGainCalc<Gnl>::_init_gain_general_net(net: &Gnl::node_t,
                                               part: &[u8]) {
    let mut num = array<usize, 2>{0U, 0U};
    for w in self.hyprgraph.gr[net].iter() {
        num[part[w]] += 1;
    }
    let weight = self.hyprgraph.get_net_weight(net);

    // #pragma unroll
    for k in {0U, 1U}.iter() {
        if num[k] == 0 {
            for w in self.hyprgraph.gr[net].iter() {
                self._modify_gain(w, -weight);
            }
        } else if num[k] == 1 {
            for w in self.hyprgraph.gr[net].iter() {
                if part[w] == k {
                    self._modify_gain(w, weight);
                    break;
                }
            }
        }
    }

    if num[0] > 0 && num[1] > 0 {
        self.totalcost += weight;
    }
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info
 * @param[out] w
 * @return i32
 */
template <Gnl>
pub fn FMBiGainCalc<Gnl>::update_move_2pin_net(part: &[u8],
                                             move_info: &MoveInfo<Gnl::node_t>) ->
    Gnl::node_t {
    let mut net_cur = self.hyprgraph.gr[move_info.net].begin();
    let mut w = (*net_cur != move_info.v) ? *net_cur : *++net_cur;
    let weight = self.hyprgraph.get_net_weight(move_info.net);
    const i32 delta = (part[w] == move_info.from_part) ? weight : -weight;
    self.delta_gain_w = 2 * delta;
    return w;
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info
 * @return ret_info
 */
template <Gnl>
void FMBiGainCalc<Gnl>::init_idx_vec(v: &Gnl::node_t, net: &Gnl::node_t) {
    // let mut rng = self.hyprgraph.gr[net] |
    //         ranges::views::remove_if([&](let mut w) { return w == v; });
    // using namespace transrangers;
    // let mut rng = filter([&](let & w) { return w != v; }, all(self.hyprgraph.gr[net]));
    // self.idx_vec = FMPmr::Vec<Gnl::node_t>(rng.begin(), rng.end(), &self.rsrc);

    self.idx_vec.clear();
    let mut rng = self.hyprgraph.gr[net];
    self.idx_vec.reserve(rng.len() - 1);
    for w in rng.iter() {
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
template <Gnl>
pub fn FMBiGainCalc<Gnl>::update_move_3pin_net(part: &[u8],
                                             move_info: &MoveInfo<Gnl::node_t>)
    -> Vec<i32> {
    // let & [net, v, from_part, _] = move_info;
    let mut num = array<usize, 2>{0U, 0U};
    for w in self.idx_vec.iter() {
        num[part[w]] += 1;
    }
    // for (let & w : self.hyprgraph.gr[move_info.net])
    // {
    //     if (w == move_info.v)
    //     {
    //         continue;
    //     }
    //     num[part[w]] += 1;
    //     idx_vec.push(w);
    // }
    let mut delta_gain = Vec<i32>{0, 0};
    let mut weight = self.hyprgraph.get_net_weight(move_info.net);
    let part_w = part[self.idx_vec[0]];

    if part_w != move_info.from_part {
        weight = -weight;
    }
    if part_w == part[self.idx_vec[1]] {
        delta_gain[0] += weight;
        delta_gain[1] += weight;
    } else {
        delta_gain[0] += weight;
        delta_gain[1] -= weight;
    }
    return delta_gain;
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info
 * @return ret_info
 */
template <Gnl>
pub fn FMBiGainCalc<Gnl>::update_move_general_net(part: &[u8],
                                                move_info: &MoveInfo<Gnl::node_t>)
    -> Vec<i32> {
    // let & [net, v, from_part, to_part] = move_info;
    let mut num = array<u8, 2>{0, 0};
    // let mut idx_vec = Vec<Gnl::node_t> {};
    for w in self.idx_vec.iter() {
        num[part[w]] += 1;
    }

    // for (let & w : self.hyprgraph.gr[move_info.net])
    // {
    //     if (w == move_info.v)
    //     {
    //         continue;
    //     }
    //     num[part[w]] += 1;
    //     idx_vec.push(w);
    // }
    let degree = self.idx_vec.len();
    let mut delta_gain = Vec<i32>(degree, 0);
    let mut weight = self.hyprgraph.get_net_weight(move_info.net);

    // #pragma unroll
    for l in {move_info.from_part, move_info.to_part}.iter() {
        if num[l] == 0 {
            for (index: usize = 0U; index != degree; ++index) {
                delta_gain[index] -= weight;
            }
        } else if num[l] == 1 {
            for (index: usize = 0U; index != degree; ++index) {
                let mut part_w = part[self.idx_vec[index]];
                if part_w == l {
                    delta_gain[index] += weight;
                    break;
                }
            }
        }
        weight = -weight;
    }
    return delta_gain;
}

// instantiation

#include <py2cpp/set.hpp>              // for set
#include <xnetwork/classes/graph.hpp>  // for Graph

#include "ckpttn/netlist.hpp"  // for Netlist, SimpleNetlist

template class FMBiGainCalc<SimpleNetlist>;