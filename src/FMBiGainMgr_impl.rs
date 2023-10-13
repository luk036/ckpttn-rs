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
template <Gnl> pub fn FMBiGainMgr<Gnl>::init(&mut self, part: &[u8]) -> i32 {
    let mut totalcost = Base::init(part);
    for bckt in self.gainbucket.iter_mut() {
        bckt.clear();
    }

    for v in self.hyprgraph.iter() {
        vlink: &mut auto = self.gain_calc.vertex_list[v];
        // let mut to_part = 1 - part[v];
        self.gainbucket[1 - part[v]].append_direct(vlink);
    }
    for v in self.hyprgraph.module_fixed.iter() {
        self.lock_all(part[v], v);
    }
    return totalcost;
}

// instantiation

#include "ckpttn/netlist.hpp"  // for Netlist, SimpleNetlist

template class FMBiGainMgr<SimpleNetlist>;