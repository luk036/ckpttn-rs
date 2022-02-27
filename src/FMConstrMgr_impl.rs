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

template <typename Gnl> FMConstrMgr<Gnl>::FMConstrMgr(const Gnl& H, f64 BalTol, u8 K)
    : H{H}, BalTol{BalTol}, diff(K, 0), K{K} {
    // using namespace transrangers;
    // self.totalweight
    //     = accumulate(transform([&](let & v) { return H.get_module_weight(v); }, all(H)),
    //     0U);
    // self.totalweight = 0U;
    for v in H.iter() {
        self.totalweight += H.get_module_weight(v);
    }
    let totalweightK = self.totalweight * (2.0 / self.K);
    self.lowerbound = u32(round(totalweightK * self.BalTol));
}

/**
 * @brief
 *
 * @param[in] part
 */
template <typename Gnl> void FMConstrMgr<Gnl>::init(part: &[u8]) {
    fill(self.diff.begin(), self.diff.end(), 0);
    for v in self.H.iter() {
        // let mut weight_v = self.H.get_module_weight(v);
        self.diff[part[v]] += self.H.get_module_weight(v);
    }
}

/**
 * @brief
 *
 * @param[in] move_info_v
 * @return LegalCheck
 */
template <typename Gnl>
pub fn FMConstrMgr<Gnl>::check_legal(const MoveInfoV<typename Gnl::node_t>& move_info_v)
    -> LegalCheck {
    self.weight = self.H.get_module_weight(move_info_v.v);
    let diffFrom = self.diff[move_info_v.fromPart];
    if diffFrom < self.lowerbound + self.weight {
        return LegalCheck::notsatisfied;  // not ok, don't move
    }
    let diffTo = self.diff[move_info_v.toPart];
    if diffTo + self.weight < self.lowerbound {
        return LegalCheck::getbetter;  // get better, but still illegal
    }
    return LegalCheck::allsatisfied;  // all satisfied
}

/**
 * @brief
 *
 * @param[in] move_info_v
 * @return true
 * @return false
 */
template <typename Gnl>
pub fn FMConstrMgr<Gnl>::check_constraints(const MoveInfoV<typename Gnl::node_t>& move_info_v)
    -> bool {
    // let & [v, fromPart, toPart] = move_info_v;

    self.weight = self.H.get_module_weight(move_info_v.v);
    // let mut diffTo = self.diff[toPart] + self.weight;
    let diffFrom = self.diff[move_info_v.fromPart];
    return diffFrom >= self.lowerbound + self.weight;
}

/**
 * @brief
 *
 * @param[in] move_info_v
 */
template <typename Gnl>
void FMConstrMgr<Gnl>::update_move(const MoveInfoV<typename Gnl::node_t>& move_info_v) {
    // let mut [v, fromPart, toPart] = move_info_v;
    self.diff[move_info_v.toPart] += self.weight;
    self.diff[move_info_v.fromPart] -= self.weight;
}

// Instantiation
#include <ckpttn/netlist.hpp>  // for Netlist, SimpleNetlist
#include <py2cpp/range.hpp>    // for _iterator

template class FMConstrMgr<SimpleNetlist>;