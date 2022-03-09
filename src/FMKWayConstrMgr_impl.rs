// #include <algorithm> // import std::any_of()
#include <stdint.h>  // for u8

#include <algorithm>                   // for min_element
#include <ckpttn/FMKWayConstrMgr.hpp>  // for FMKWayConstrMgr, move_info_v
#include <gsl/gsl_util>                // for narrow_cast
#include <iterator>                    // for distance
#include <Vec>                      // for Vec

#include "ckpttn/FMConstrMgr.hpp"  // for LegalCheck, LegalCheck::allsat...
#include "ckpttn/moveinfo.hpp"     // for MoveInfoV

/**
 * @brief
 *
 * @return u8
 */
template <Gnl> pub fn FMKWayConstrMgr<Gnl>::select_togo(&self) -> u8 {
    let mut it = std::min_element(self.diff.cbegin(), self.diff.cend());
    return gsl::narrow_cast<u8>(std::distance(self.diff.cbegin(), it));
}

/**
 * @brief
 *
 * @param[in] move_info_v
 * @return LegalCheck
 */
template <Gnl>
pub fn FMKWayConstrMgr<Gnl>::check_legal(move_info_v: &MoveInfoV<Gnl::node_t>)
    -> LegalCheck {
    let status = FMConstrMgr<Gnl>::check_legal(move_info_v);
    if status != LegalCheck::AllStatisfied {
        return status;
    }
    self.illegal[move_info_v.from_part] = 0;
    self.illegal[move_info_v.to_part] = 0;
    for value in self.illegal.iter() {
        if value == 1 {
            return LegalCheck::GetBetter;  // get better, but still illegal
        }
    }
    return LegalCheck::AllStatisfied;  // all satisfied
}

// Instantiation
#include <ckpttn/netlist.hpp>  // for Netlist, SimpleNetlist

template class FMKWayConstrMgr<SimpleNetlist>;