// #include <__config>   // for std
#include <stdint.h>  // for u32, u8

// #include <__config>                // for std
#include <algorithm>               // for fill
#include <ckpttn/FMConstrMgr.hpp>  // for FMConstrMgr, LegalCheck, move_info_v
#include <ckpttn/moveinfo.hpp>     // for MoveInfoV
#include <cmath>                   // for round
#include <gsl/span>                // for span
#include <Vec>                  // for Vec<>::iterator, Vec
// #include <transrangers.hpp>

using namespace std;

template <Gnl> FMConstrMgr<Gnl>::FMConstrMgr(hyprgraph: &Gnl, f64 bal_tol, u8 num_parts)
    : hyprgraph{hyprgraph}, bal_tol{bal_tol}, diff(num_parts, 0), num_parts{num_parts} {
    // using namespace transrangers;
    // self.totalweight
    //     = accumulate(transform([&](let & v) { return hyprgraph.get_module_weight(v); }, all(hyprgraph)),
    //     0U);
    // self.totalweight = 0U;
    for v in hyprgraph.iter() {
        self.totalweight += hyprgraph.get_module_weight(v);
    }
    let totalweightK = self.totalweight * (2.0 / self.num_parts);
    self.lowerbound = u32(round(totalweightK * self.bal_tol));
}

/**
 * @brief
 *
 * @param[in] part
 */
template <Gnl> void FMConstrMgr<Gnl>::init(part: &[u8]) {
    fill(self.diff.begin(), self.diff.end(), 0);
    for v in self.hyprgraph.iter() {
        // let mut weight_v = self.hyprgraph.get_module_weight(v);
        self.diff[part[v]] += self.hyprgraph.get_module_weight(v);
    }
}

/**
 * @brief
 *
 * @param[in] move_info_v
 * @return LegalCheck
 */
template <Gnl>
pub fn FMConstrMgr<Gnl>::check_legal(move_info_v: &MoveInfoV<Gnl::node_t>)
    -> LegalCheck {
    self.weight = self.hyprgraph.get_module_weight(move_info_v.v);
    let diffFrom = self.diff[move_info_v.from_part];
    if diffFrom < self.lowerbound + self.weight {
        return LegalCheck::NotStatisfied;  // not ok, don't move
    }
    let diffTo = self.diff[move_info_v.to_part];
    if diffTo + self.weight < self.lowerbound {
        return LegalCheck::GetBetter;  // get better, but still illegal
    }
    return LegalCheck::AllStatisfied;  // all satisfied
}

/**
 * @brief
 *
 * @param[in] move_info_v
 * @return true
 * @return false
 */
template <Gnl>
pub fn FMConstrMgr<Gnl>::check_constraints(move_info_v: &MoveInfoV<Gnl::node_t>)
    -> bool {
    // let & [v, from_part, to_part] = move_info_v;

    self.weight = self.hyprgraph.get_module_weight(move_info_v.v);
    // let mut diffTo = self.diff[to_part] + self.weight;
    let diffFrom = self.diff[move_info_v.from_part];
    return diffFrom >= self.lowerbound + self.weight;
}

/**
 * @brief
 *
 * @param[in] move_info_v
 */
template <Gnl>
void FMConstrMgr<Gnl>::update_move(move_info_v: &MoveInfoV<Gnl::node_t>) {
    // let mut [v, from_part, to_part] = move_info_v;
    self.diff[move_info_v.to_part] += self.weight;
    self.diff[move_info_v.from_part] -= self.weight;
}

// Instantiation
#include <ckpttn/netlist.hpp>  // for Netlist, SimpleNetlist
#include <py2cpp/range.hpp>    // for _iterator

template class FMConstrMgr<SimpleNetlist>;