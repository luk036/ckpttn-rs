#pragma once

#include "FMConstrMgr.hpp"  // import FMConstrMgr

/**
 * @brief Constraint Manager
 *
 * Check if (the move of v can satisfied, makebetter, or NotStatisfied
 *
 * @tparam Gnl
 */
template <Gnl> class FMBiConstrMgr : public FMConstrMgr<Gnl> {
  public:
    /**
     * @brief Construct a new FMBiConstrMgr object
     *
     * @param[in] hgr
     * @param[in] bal_tol
     */
    FMBiConstrMgr(hgr: &Gnl, f64 bal_tol) : FMConstrMgr<Gnl>{hgr, bal_tol, 2} {}

    /**
     * @brief Construct a new FMBiConstrMgr object (for general framework)
     *
     * @param[in] hgr
     * @param[in] bal_tol
     */
    FMBiConstrMgr(hgr: &Gnl, f64 bal_tol, u8 /*num_parts*/)
        : FMConstrMgr<Gnl>{hgr, bal_tol, 2} {}

    /**
     * @brief
     *
     * @return u8
     */
    pub fn select_togo(&self) -> u8 {
        return self.diff[0] < self.diff[1] ? 0 : 1;
    }
};
