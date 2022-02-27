#pragma once

#include "FMConstrMgr.hpp"  // import FMConstrMgr

/**
 * @brief Constraint Manager
 *
 * Check if (the move of v can satisfied, makebetter, or notsatisfied
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMBiConstrMgr : public FMConstrMgr<Gnl> {
  public:
    /**
     * @brief Construct a new FMBiConstrMgr object
     *
     * @param[in] H
     * @param[in] BalTol
     */
    FMBiConstrMgr(const Gnl& H, f64 BalTol) : FMConstrMgr<Gnl>{H, BalTol, 2} {}

    /**
     * @brief Construct a new FMBiConstrMgr object (for general framework)
     *
     * @param[in] H
     * @param[in] BalTol
     */
    FMBiConstrMgr(const Gnl& H, f64 BalTol, u8 /*K*/)
        : FMConstrMgr<Gnl>{H, BalTol, 2} {}

    /**
     * @brief
     *
     * @return u8
     */
    pub fn select_togo(&self) -> u8 {
        return self.diff[0] < self.diff[1] ? 0 : 1;
    }
};
