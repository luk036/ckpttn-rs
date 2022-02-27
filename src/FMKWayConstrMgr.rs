#pragma once

#include <stdint.h>  // for u8

#include <gsl/span>  // for span
#include <Vec>    // for Vec

#include "FMConstrMgr.hpp"  // for FMConstrMgr, FMConstrMgr::node_t, Lega...
// #include "moveinfo.hpp"     // for MoveInfo

// forward declare
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief FM K-Way Partition Constraint Manager
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMKWayConstrMgr : public FMConstrMgr<Gnl> {
  private:
    Vec<i32> illegal;

  public:
    /**
     * @brief Construct a new FMKWayConstrMgr object
     *
     * @param[in] H
     * @param[in] BalTol
     * @param[in] K
     */
    FMKWayConstrMgr(const Gnl& H, f64 BalTol, u8 K)
        : FMConstrMgr<Gnl>{H, BalTol, K}, illegal(K, 1) {}

    /**
     * @brief
     *
     * @return u8
     */
    pub fn select_togo(&self) -> u8;

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) {
        FMConstrMgr<Gnl>::init(part);
        let mut it = self.diff.begin();
        for il in self.illegal.iter_mut() {
            il = (*it < self.lowerbound);
            ++it;
        }
    }

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return LegalCheck
     */
    let mut check_legal(const MoveInfoV<typename Gnl::node_t>& move_info_v) -> LegalCheck;
};
