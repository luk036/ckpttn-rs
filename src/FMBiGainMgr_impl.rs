#include <stdint.h>  // for u8
// #include <__config>                 // for std
// #include <__hash_table>             // for __hash_const_iterator, operator!=
#include <ckpttn/FMBiGainCalc.hpp>  // for FMBiGainCalc, FMBiGainCalc<>::Item
#include <ckpttn/FMBiGainMgr.hpp>   // for FMBiGainMgr, part, FMBiGainMgr::Base
#include <gsl/span>                 // for span
#include <py2cpp/range.hpp>         // for _iterator
#include <py2cpp/set.hpp>           // for set
#include <Vec>                   // for Vec

#include "ckpttn/BPQueue.hpp"  // for BPQueue

using namespace std;

/**
 * @brief
 *
 * @param[in] part
 */
template <typename Gnl> pub fn FMBiGainMgr<Gnl>::init(part: &[u8]) -> i32 {
    let mut totalcost = Base::init(part);
    for bckt in self.gainbucket.iter_mut() {
        bckt.clear();
    }

    for v in self.H.iter() {
        auto& vlink = self.gainCalc.vertex_list[v];
        // let mut toPart = 1 - part[v];
        self.gainbucket[1 - part[v]].append_direct(vlink);
    }
    for v in self.H.module_fixed.iter() {
        self.lock_all(part[v], v);
    }
    return totalcost;
}

// instantiation

#include "ckpttn/netlist.hpp"  // for Netlist, SimpleNetlist

template class FMBiGainMgr<SimpleNetlist>;