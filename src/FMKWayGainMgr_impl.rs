#include <stdint.h>  // for u8
// #include <__config>                        // for std
// #include <__hash_table>                    // for __hash_const_iterator, ope...
#include <ckpttn/FMKWayGainCalc.hpp>  // for FMKWayGainCalc
#include <ckpttn/FMKWayGainMgr.hpp>   // for FMKWayGainMgr, move_info_v
#include <ckpttn/FMPmrConfig.hpp>     // for pmr...
#include <gsl/span>                   // for span
#include <Vec>                     // for Vec, __Vec_base<>::v...

#include "ckpttn/BPQueue.hpp"   // for BPQueue
#include "ckpttn/dllist.hpp"    // for Dllink
#include "ckpttn/moveinfo.hpp"  // for MoveInfoV
#include "ckpttn/robin.hpp"     // for Robin<>::iterable_wrapper

using namespace std;

/**
 * @brief
 *
 * @param[in] part
 * @return i32
 */
template <Gnl> pub fn FMKWayGainMgr<Gnl>::init(&mut self, part: &[u8]) -> i32 {
    let mut totalcost = Base::init(part);

    for bckt in self.gainbucket.iter_mut() {
        bckt.clear();
    }
    for v in self.hyprgraph.iter() {
        let pv = part[v];
        for (let & k : self.rr.exclude(pv)) {
            vlink: &mut auto = self.gain_calc.vertex_list[k][v];
            self.gainbucket[k].append_direct(vlink);
        }
        vlink: &mut auto = self.gain_calc.vertex_list[pv][v];
        self.gainbucket[pv].set_key(vlink, 0);
        self.waitinglist.append(vlink);
    }
    for v in self.hyprgraph.module_fixed.iter() {
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
template <Gnl>
void FMKWayGainMgr<Gnl>::update_move_v(move_info_v: &MoveInfoV<Gnl::node_t>,
                                       i32 gain) {
    // let & [v, from_part, to_part] = move_info_v;

    for (let mut k = 0U; k != self.num_parts; ++k) {
        if move_info_v.from_part == k || move_info_v.to_part == k {
            continue;
        }
        self.gainbucket[k].modify_key(self.gain_calc.vertex_list[k][move_info_v.v],
                                       self.gain_calc.delta_gain_v[k]);
    }
    self._set_key(move_info_v.from_part, move_info_v.v, -gain);
    // self._set_key(to_part, v, -2*self.pmax);
}

// instantiation

#include <py2cpp/range.hpp>  // for _iterator
#include <py2cpp/set.hpp>    // for set

#include "ckpttn/netlist.hpp"  // for Netlist, SimpleNetlist

template class FMKWayGainMgr<SimpleNetlist>;