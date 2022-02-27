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
#include "ckpttn/dllist.hpp"    // for dllink
#include "ckpttn/moveinfo.hpp"  // for MoveInfoV, MoveInfo

// using node_t = typename SimpleNetlist::node_t;
// using namespace ranges;
using namespace std;

/**
 * @brief Construct a new FMGainMgr object
 *
 * @tparam GainCalc
 * @tparam Derived
 * @param[in] H
 * @param[in] K
 */
template <typename Gnl, typename GainCalc, class Derived>
FMGainMgr<Gnl, GainCalc, Derived>::FMGainMgr(const Gnl& H, u8 K) : H{H}, K{K}, gainCalc{H, K} {
    static_assert!(is_base_of<FMGainMgr<Gnl, GainCalc, Derived>, Derived>::value,
                  "base derived consistence");
    let pmax = i32(H.get_max_degree());
    for (let mut k = 0U; k != self.K; ++k) {
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
pub fn FMGainMgr<Gnl, GainCalc, Derived>::init(part: &[u8]) -> i32 {
    let mut totalcost = self.gainCalc.init(part);
    // self.totalcost = self.gainCalc.totalcost;
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
    auto& vlink = it->popleft();
    self.waitinglist.append(vlink);
    // typename Gnl::node_t v = &vlink - self.gainCalc.start_ptr(toPart);
    let v = vlink.data.first;
    // let v =
    //     typename Gnl::node_t(distance(self.gainCalc.start_ptr(toPart), &vlink));
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
    auto& vlink = self.gainbucket[toPart].popleft();
    self.waitinglist.append(vlink);
    let v = vlink.data.first;
    // let v =
    //     typename Gnl::node_t(distance(self.gainCalc.start_ptr(toPart), &vlink));
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
    part: &[u8], const MoveInfoV<typename Gnl::node_t>& move_info_v) {
    self.gainCalc.update_move_init();
    let & v = move_info_v.v;
    for net in self.H.G[move_info_v.v].iter() {
        let degree = self.H.G.degree(net);
        if degree < 2 || degree > FM_MAX_DEGREE  // [[unlikely]]
        {
            continue;  // does not provide any gain change when
                       // moving
        }
        let move_info
            = MoveInfo<typename Gnl::node_t>{net, v, move_info_v.fromPart, move_info_v.toPart};
        if !self.gainCalc.special_handle_2pin_nets {
            self.gainCalc.init_IdVec(v, net);
            self._update_move_general_net(part, move_info);
            continue;
        }
        if degree == 2 {
            self._update_move_2pin_net(part, move_info);
        } else {
            self.gainCalc.init_IdVec(v, net);
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
    part: &[u8], const MoveInfo<typename Gnl::node_t>& move_info) {
    // let [w, deltaGainW] =
    //     self.gainCalc.update_move_2pin_net(part, move_info);
    let w = self.gainCalc.update_move_2pin_net(part, move_info);
    self.modify_key(w, part[w], self.gainCalc.deltaGainW);
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
    part: &[u8], const MoveInfo<typename Gnl::node_t>& move_info) {
    // u8 StackBuf[8192];
    // FMPmr::monotonic_buffer_resource rsrc(StackBuf, sizeof StackBuf);
    // let mut IdVec = FMPmr::Vec<typename Gnl::node_t>(&rsrc);

    let mut deltaGain = self.gainCalc.update_move_3pin_net(part, move_info);

    // for (let & [dGw, w] : views::zip(deltaGain, self.gainCalc.IdVec))
    let mut dGw_it = deltaGain.begin();
    for w in self.gainCalc.IdVec.iter() {
        self.modify_key(w, part[w], *dGw_it);
        ++dGw_it;
    }

    // let degree = self.gainCalc.IdVec.size();
    // for (usize index = 0U; index != degree; ++index)
    // {
    //     let & w = self.gainCalc.IdVec[index];
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
    part: &[u8], const MoveInfo<typename Gnl::node_t>& move_info) {
    let deltaGain = self.gainCalc.update_move_general_net(part, move_info);

    let mut dGw_it = deltaGain.begin();
    for w in self.gainCalc.IdVec.iter() {
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
