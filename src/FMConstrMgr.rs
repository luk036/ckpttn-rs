#pragma once

#include <cinttypes>  // for u8
#include <gsl/span>   // for span
#include <Vec>     // for Vec

// #include "moveinfo.hpp"  // for MoveInfo

// forward declare
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief Check if the move of v can satisfied, GetBetter, or NotStatisfied
 *
 */
enum class LegalCheck { NotStatisfied, GetBetter, AllStatisfied };

/**
 * @brief FM Partition Constraint Manager
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMConstrMgr {
  private:
    hgr: &Gnl
    f64 bal_tol;
    u32 totalweight{0};
    u32 weight{};  // cache value

  protected:
    Vec<u32> diff;
    u32 lowerbound{};
    u8 num_parts;

    using node_t = typename Gnl::node_t;

    /**
     * @brief Construct a new FMConstrMgr object
     *
     * @param[in] hgr
     * @param[in] bal_tol
     */
    FMConstrMgr(hgr: &Gnl, f64 bal_tol) : FMConstrMgr(hgr, bal_tol, 2) {}

    /**
     * @brief Construct a new FMConstrMgr object
     *
     * @param[in] hgr
     * @param[in] bal_tol
     * @param[in] num_parts
     */
    FMConstrMgr(hgr: &Gnl, f64 bal_tol, u8 num_parts);

  public:
    /**
     * @brief
     *
     * @param[in] part
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return LegalCheck
     */
    pub fn check_legal(&mut self, move_info_v: &MoveInfoV<node_t>) -> LegalCheck;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return true
     * @return false
     */
    pub fn check_constraints(&mut self, move_info_v: &MoveInfoV<node_t>) -> bool;

    /**
     * @brief
     *
     * @param[in] move_info_v
     */
    pub fn update_move(&mut self, move_info_v: &MoveInfoV<node_t>) -> void;
};
