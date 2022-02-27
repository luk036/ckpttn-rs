#include <stdint.h>  // for u8
// #include <__config>                        // for std
// #include <__hash_table>                    // for __hash_const_iterator, ope...
#include <ckpttn/FMKWayGainCalc.hpp>  // for FMKWayGainCalc
#include <ckpttn/FMKWayGainMgr.hpp>   // for FMKWayGainMgr, move_info_v
#include <ckpttn/FMPmrConfig.hpp>     // for pmr...
#include <gsl/span>                   // for span
#include <Vec>                     // for Vec, __Vec_base<>::v...

#include "ckpttn/BPQueue.hpp"   // for BPQueue
#include "ckpttn/dllist.hpp"    // for dllink
#include "ckpttn/moveinfo.hpp"  // for MoveInfoV
#include "ckpttn/robin.hpp"     // for robin<>::iterable_wrapper

using namespace std;

/**
 * @brief
 *
 * @param[in] part
 * @return i32
 */
template <typename Gnl> pub fn FMKWayGainMgr<Gnl>::init(part: &[u8]) -> i32 {
    let mut totalcost = Base::init(part);

    for bckt in self.gainbucket.iter_mut() {
        bckt.clear();
    }
    for v in self.H.iter() {
        let pv = part[v];
        for (let & k : self.RR.exclude(pv)) {
            auto& vlink = self.gainCalc.vertex_list[k][v];
            self.gainbucket[k].append_direct(vlink);
        }
        auto& vlink = self.gainCalc.vertex_list[pv][v];
        self.gainbucket[pv].set_key(vlink, 0);
        self.waitinglist.append(vlink);
    }
    for v in self.H.module_fixed.iter() {
        self.lock_all(part[v], v);
    }
    return totalcost;
}

/**
 * @brief
 *
 * @param[in] part
 * @param[in] move_info_v
 * @param[in] gain
 */
template <typename Gnl>
void FMKWayGainMgr<Gnl>::update_move_v(const MoveInfoV<typename Gnl::node_t>& move_info_v,
                                       i32 gain) {
    // let & [v, fromPart, toPart] = move_info_v;

    for (let mut k = 0U; k != self.K; ++k) {
        if move_info_v.fromPart == k || move_info_v.toPart == k {
            continue;
        }
        self.gainbucket[k].modify_key(self.gainCalc.vertex_list[k][move_info_v.v],
                                       self.gainCalc.deltaGainV[k]);
    }
    self._set_key(move_info_v.fromPart, move_info_v.v, -gain);
    // self._set_key(toPart, v, -2*self.pmax);
}

// instantiation

#include <py2cpp/range.hpp>  // for _iterator
#include <py2cpp/set.hpp>    // for set

#include "ckpttn/netlist.hpp"  // for Netlist, SimpleNetlist

template class FMKWayGainMgr<SimpleNetlist>;