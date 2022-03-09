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

// using node_t = typename SimpleNetlist::node_t;
// using namespace ranges;
using namespace std;

/**
 * @brief Construct a new FMGainMgr object
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] hgr
 * @param[in] num_parts
 */
template <typename Gnl, typename GainCalc, class Derived>
FMGainMgr<Gnl, GainCalc, Derived>::FMGainMgr(hgr: &Gnl, u8 num_parts) : hgr{hgr}, num_parts{num_parts}, gain_calc{hgr, num_parts} {
    static_assert!(is_base_of<FMGainMgr<Gnl, GainCalc, Derived>, Derived>::value,
                  "base derived consistence");
    let pmax = hgr.get_max_degree();
    for (let mut k = 0U; k != self.num_parts; ++k) {
        self.gainbucket.emplace_back(BPQueue<typename Gnl::node_t>(-pmax, pmax));
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
template <typename Gnl, typename GainCalc, class Derived>
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
 * @return tuple<MoveInfoV<typename Gnl::node_t>, i32>
 */
template <typename Gnl, typename GainCalc, class Derived>
pub fn FMGainMgr<Gnl, GainCalc, Derived>::select(part: &[u8])
    -> tuple<MoveInfoV<typename Gnl::node_t>, i32> {
    let it = max_element(
        self.gainbucket.begin(), self.gainbucket.end(),
        [](let & bckt1, let & bckt2) { return bckt1.get_max() < bckt2.get_max(); });

    let toPart = gsl::narrow_cast<u8>(distance(self.gainbucket.begin(), it));
    let gainmax = it->get_max();
    vlink: &mut auto = it->popleft();
    self.waitinglist.append(vlink);
    // typename Gnl::node_t v = &vlink - self.gain_calc.start_ptr(toPart);
    let v = vlink.data.first;
    // let v =
    //     typename Gnl::node_t(distance(self.gain_calc.start_ptr(toPart), &vlink));
    // let mut move_info_v = MoveInfoV<typename Gnl::node_t> {v, part[v], toPart};
    return {{v, part[v], toPart}, gainmax};
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] toPart
 * @return tuple<typename Gnl::node_t, i32>
 */
template <typename Gnl, typename GainCalc, class Derived>
pub fn FMGainMgr<Gnl, GainCalc, Derived>::select_togo(u8 toPart)
    -> tuple<typename Gnl::node_t, i32> {
    let gainmax = self.gainbucket[toPart].get_max();
    vlink: &mut auto = self.gainbucket[toPart].popleft();
    self.waitinglist.append(vlink);
    let v = vlink.data.first;
    // let v =
    //     typename Gnl::node_t(distance(self.gain_calc.start_ptr(toPart), &vlink));
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
template <typename Gnl, typename GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::update_move(
    part: &[u8], move_info_v: &MoveInfoV<typename Gnl::node_t>) {
    self.gain_calc.update_move_init();
    let & v = move_info_v.v;
    for net in self.hgr.gr[move_info_v.v].iter() {
        let degree = self.hgr.gr.degree(net);
        if degree < 2 || degree > FM_MAX_DEGREE  // [[unlikely]]
        {
            continue;  // does not provide any gain change when
                       // moving
        }
        let move_info
            = MoveInfo<typename Gnl::node_t>{net, v, move_info_v.fromPart, move_info_v.toPart};
        if !self.gain_calc.special_handle_2pin_nets {
            self.gain_calc.init_IdVec(v, net);
            self._update_move_general_net(part, move_info);
            continue;
        }
        if degree == 2 {
            self._update_move_2pin_net(part, move_info);
        } else {
            self.gain_calc.init_IdVec(v, net);
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
template <typename Gnl, typename GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_2pin_net(
    part: &[u8], move_info: &MoveInfo<typename Gnl::node_t>) {
    // let [w, deltaGainW] =
    //     self.gain_calc.update_move_2pin_net(part, move_info);
    let w = self.gain_calc.update_move_2pin_net(part, move_info);
    self.modify_key(w, part[w], self.gain_calc.deltaGainW);
}

/**
 * @brief
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] part
 * @param[in] move_info
 */
template <typename Gnl, typename GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_3pin_net(
    part: &[u8], move_info: &MoveInfo<typename Gnl::node_t>) {
    // u8 StackBuf[8192];
    // FMPmr::monotonic_buffer_resource rsrc(StackBuf, sizeof StackBuf);
    // let mut IdVec = FMPmr::Vec<typename Gnl::node_t>(&rsrc);

    let mut deltaGain = self.gain_calc.update_move_3pin_net(part, move_info);

    // for (let & [dGw, w] : views::zip(deltaGain, self.gain_calc.IdVec))
    let mut dGw_it = deltaGain.begin();
    for w in self.gain_calc.IdVec.iter() {
        self.modify_key(w, part[w], *dGw_it);
        ++dGw_it;
    }

    // let degree = self.gain_calc.IdVec.size();
    // for (index: usize = 0U; index != degree; ++index)
    // {
    //     let & w = self.gain_calc.IdVec[index];
    //     self.modify_key(w, part[w], deltaGain[index]);
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
template <typename Gnl, typename GainCalc, class Derived>
void FMGainMgr<Gnl, GainCalc, Derived>::_update_move_general_net(
    part: &[u8], move_info: &MoveInfo<typename Gnl::node_t>) {
    let deltaGain = self.gain_calc.update_move_general_net(part, move_info);

    let mut dGw_it = deltaGain.begin();
    for w in self.gain_calc.IdVec.iter() {
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
