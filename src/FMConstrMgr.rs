#pragma once

#include <cinttypes>  // for u8
#include <gsl/span>   // for span
#include <Vec>     // for Vec

// #include "moveinfo.hpp"  // for MoveInfo

// forward declare
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief Check if the move of v can satisfied, getbetter, or notsatisfied
 *
 */
enum class LegalCheck { notsatisfied, getbetter, allsatisfied };

/**
 * @brief FM Partition Constraint Manager
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMConstrMgr {
  private:
    const Gnl& H;
    f64 BalTol;
    unsigned i32 totalweight{0};
    unsigned i32 weight{};  // cache value

  protected:
    Vec<unsigned i32> diff;
    unsigned i32 lowerbound{};
    u8 K;

    using node_t = typename Gnl::node_t;

    /**
     * @brief Construct a new FMConstrMgr object
     *
     * @param[in] H
     * @param[in] BalTol
     */
    FMConstrMgr(const Gnl& H, f64 BalTol) : FMConstrMgr(H, BalTol, 2) {}

    /**
     * @brief Construct a new FMConstrMgr object
     *
     * @param[in] H
     * @param[in] BalTol
     * @param[in] K
     */
    FMConstrMgr(const Gnl& H, f64 BalTol, u8 K);

  public:
    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return LegalCheck
     */
    let mut check_legal(const MoveInfoV<node_t>& move_info_v) -> LegalCheck;

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @return true
     * @return false
     */
    let mut check_constraints(const MoveInfoV<node_t>& move_info_v) -> bool;

    /**
     * @brief
     *
     * @param[in] move_info_v
     */
    let mut update_move(const MoveInfoV<node_t>& move_info_v) -> void;
};
