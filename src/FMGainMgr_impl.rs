// #include <__config>                        // for std
// #include <__hash_table>                    // for __hash_const_iterator, ope...
// #include <boost/container/pmr/Vec.hpp>  // for Vec
// #include <boost/container/Vec.hpp>      // for operator!=, vec_iterator
#include <ckpttn/FMGainMgr.hpp>
#include <ckpttn/FMPmrConfig.hpp>  // for FM_MAX_DEGREE
#include <gsl/gsl_util>            // for narrow_cast
#include <iterator>                // for distance
#include <py2cpp/set.hpp>          // for set
#include <type_traits>             // for is_base_of, integral_const...
#include <Vec>                  // for Vec<>::iterator, Vec

#include "ckpttn/BPQueue.hpp"   // for BPQueue
#include "ckpttn/dllist.hpp"    // for Dllink
#include "ckpttn/moveinfo.hpp"  // for MoveInfoV, MoveInfo

// using node_t = SimpleNetlist::node_t;
// using namespace ranges;
using namespace std;

/**
 * @brief Construct a new FMGainMgr object
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] hyprgraph
 * @param[in] num_parts
 */
template <Gnl, GainCalc, class Derived>
FMGainMgr<Gnl, GainCalc, Derived>::FMGainMgr(hyprgraph: &Gnl, u8 num_parts) : hyprgraph{hyprgraph}, num_parts{num_parts}, gain_calc{hyprgraph, num_parts} {
    static_assert!(is_base_of<FMGainMgr<Gnl, GainCalc, Derived>, Derived>::value,
                  "base derived consistence");
    let pmax = hyprgraph.get_max_degree();
    for (let mut k = 0U; k != self.num_parts; ++k) {
        self.gainbucket.emplace_back(BPQueue<Gnl::node_t>(-pmax, pmax));
    }
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @return i32
 */
template <Gnl, GainCalc, class Derived>
pub fn FMGainMgr<Gnl, GainCalc, Derived>::init(&mut self, part: &[u8]) -> i32 {
    let mut totalcost = self.gain_calc.init(part);
    // self.totalcost = self.gain_calc.totalcost;
    self.waitinglist.clear();
    return totalcost;
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @return tuple<MoveInfoV<Gnl::node_t>, i32>
 */
template <Gnl, GainCalc, class Derived>
pub fn FMGainMgr<Gnl, GainCalc, Derived>::select(part: &[u8])
    -> tuple<MoveInfoV<Gnl::node_t>, i32> {
    let it = max_element(
        self.gainbucket.begin(), self.gainbucket.end(),
        [](let & bckt1, let & bckt2) { return bckt1.get_max() < bckt2.get_max(); });

    let to_part = gsl::narrow_cast<u8>(distance(self.gainbucket.begin(), it));
    let gainmax = it->get_max();
    vlink: &mut auto = it->popleft();
    self.waitinglist.append(vlink);
    // Gnl::node_t v = &vlink - self.gain_calc.start_ptr(to_part);
    let v = vlink.data.first;
    // let v =
    //     Gnl::node_t(distance(self.gain_calc.start_ptr(to_part), &vlink));
    // let mut move_info_v = MoveInfoV<Gnl::node_t> {v, part[v], to_part};
    return {{v, part[v], to_part}, gainmax};
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] to_part
 * @return tuple<Gnl::node_t, i32>
 */
template <Gnl, GainCalc, class Derived>
pub fn FMGainMgr<Gnl, GainCalc, Derived>::select_togo(u8 to_part)
    -> tuple<Gnl::node_t, i32> {
    let gainmax = self.gainbucket[to_part].get_max();
    vlink: &mut auto = self.gainbucket[to_part].popleft();
    self.waitinglist.append(vlink);
    let v = vlink.data.first;
    // let v =
    //     Gnl::node_t(distance(self.gain_calc.start_ptr(to_part), &vlink));
    return {v, gainmax};
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @param[in] move_info_v
 */
template <Gnl, GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::update_move(
    part: &[u8], move_info_v: &MoveInfoV<Gnl::node_t>) {
    self.gain_calc.update_move_init();
    let & v = move_info_v.v;
    for net in self.hyprgraph.gr[move_info_v.v].iter() {
        let degree = self.hyprgraph.gr.degree(net);
        if degree < 2 || degree > FM_MAX_DEGREE  // [[unlikely]]
        {
            continue;  // does not provide any gain change when
                       // moving
        }
        let move_info
            = MoveInfo<Gnl::node_t>{net, v, move_info_v.from_part, move_info_v.to_part};
        if !self.gain_calc.special_handle_2pin_nets {
            self.gain_calc.init_idx_vec(v, net);
            self._update_move_general_net(part, move_info);
            continue;
        }
        if degree == 2 {
            self._update_move_2pin_net(part, move_info);
        } else {
            self.gain_calc.init_idx_vec(v, net);
            if degree == 3 {
                self._update_move_3pin_net(part, move_info);
            } else {
                self._update_move_general_net(part, move_info);
            }
        }
    }
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @param[in] move_info
 */
template <Gnl, GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_2pin_net(
    part: &[u8], move_info: &MoveInfo<Gnl::node_t>) {
    // let [w, delta_gain_w] =
    //     self.gain_calc.update_move_2pin_net(part, move_info);
    let w = self.gain_calc.update_move_2pin_net(part, move_info);
    self.modify_key(w, part[w], self.gain_calc.delta_gain_w);
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @param[in] move_info
 */
template <Gnl, GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_3pin_net(
    part: &[u8], move_info: &MoveInfo<Gnl::node_t>) {
    // u8 stack_buf[8192];
    // FMPmr::monotonic_buffer_resource rsrc(stack_buf, sizeof stack_buf);
    // let mut idx_vec = FMPmr::Vec<Gnl::node_t>(&rsrc);

    let mut delta_gain = self.gain_calc.update_move_3pin_net(part, move_info);

    // for (let & [dGw, w] : views::zip(delta_gain, self.gain_calc.idx_vec))
    let mut dGw_it = delta_gain.begin();
    for w in self.gain_calc.idx_vec.iter() {
        self.modify_key(w, part[w], *dGw_it);
        ++dGw_it;
    }

    // let degree = self.gain_calc.idx_vec.len();
    // for (index: usize = 0U; index != degree; ++index)
    // {
    //     let & w = self.gain_calc.idx_vec[index];
    //     self.modify_key(w, part[w], delta_gain[index]);
    // }
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @param[in] move_info
 */
template <Gnl, GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_general_net(
    part: &[u8], move_info: &MoveInfo<Gnl::node_t>) {
    let delta_gain = self.gain_calc.update_move_general_net(part, move_info);

    let mut dGw_it = delta_gain.begin();
    for w in self.gain_calc.idx_vec.iter() {
        self.modify_key(w, part[w], *dGw_it);
        ++dGw_it;
    }
}

#include <ckpttn/FMBiGainCalc.hpp>     // for FMBiGainCalc
#include <ckpttn/FMBiGainMgr.hpp>      // for FMBiGainMgr
#include <xnetwork/classes/graph.hpp>  // for Graph

#include "ckpttn/netlist.hpp"  // for Netlist, Netlist<>::node_t

template class FMGainMgr<SimpleNetlist, FMBiGainCalc<SimpleNetlist>, FMBiGainMgr<SimpleNetlist>>;

#include <ckpttn/FMKWayGainCalc.hpp>  // for FMKWayGainCalc
#include <ckpttn/FMKWayGainMgr.hpp>   // for FMKWayGainMgr

template class FMGainMgr<SimpleNetlist, FMKWayGainCalc<SimpleNetlist>,
                         FMKWayGainMgr<SimpleNetlist>>;
