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
 * @brief FM num_parts-Way Partition Constraint Manager
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
     * @param[in] hgr
     * @param[in] bal_tol
     * @param[in] num_parts
     */
    FMKWayConstrMgr(hgr: &Gnl, f64 bal_tol, u8 num_parts)
        : FMConstrMgr<Gnl>{hgr, bal_tol, num_parts}, illegal(num_parts, 1) {}

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
    pub fn init(gsl::span<const u8> part) {
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
    pub fn check_legal(&mut self, move_info_v: &MoveInfoV<typename Gnl::node_t>) -> LegalCheck;
};
