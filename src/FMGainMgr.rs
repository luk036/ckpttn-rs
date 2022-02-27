#pragma once

#include <algorithm>  // for all_of
#include <cinttypes>  // for u8, u32
#include <gsl/span>   // for span
#include <tuple>      // for tuple
#include <utility>    // for pair
#include <Vec>     // for Vec<>::const_iterator, Vec

#include "BPQueue.hpp"  // for BPQueue
#include "dllist.hpp"   // for dllink

template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainCalc
 * @tparam Derived
 */
template <typename Gnl, typename GainCalc, class Derived> class FMGainMgr {
    Derived& self = *static_cast<Derived*>(this);
    using node_t = typename Gnl::node_t;
    // friend Derived;
    using Item = dllink<std::pair<node_t, u32>>;

  protected:
    Item waitinglist{std::make_pair(node_t{}, u32(0))};
    const Gnl& H;
    Vec<BPQueue<node_t>> gainbucket;
    // usize pmax;
    u8 K;

  public:
    GainCalc gainCalc;

    // i32 totalcost;

    // FMGainMgr(FMGainMgr&&) = default;

    /**
     * @brief Construct a new FMGainMgr object
     *
     * @param[in] H
     * @param[in] K
     */
    FMGainMgr(const Gnl& H, u8 K);

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) -> i32;

    /**
     * @brief
     *
     * @param[in] toPart
     * @return true
     * @return false
     */
    pub fn is_empty_togo(u8 toPart) const -> bool {
        return self.gainbucket[toPart].is_empty();
    }

    /**
     * @brief
     *
     * @return true
     * @return false
     */
    pub fn is_empty(&self) -> bool {
        return std::all_of(self.gainbucket.cbegin(), self.gainbucket.cend(),
                           [&](let & bckt) { return bckt.is_empty(); });
    }

    /**
     * @brief
     *
     * @param[in] part
     * @return std::tuple<MoveInfoV<node_t>, i32>
     */
    let mut select(gsl::span<const u8> part) -> std::tuple<MoveInfoV<node_t>, i32>;

    /**
     * @brief
     *
     * @param[in] toPart
     * @return std::tuple<node_t, i32>
     */
    let mut select_togo(u8 toPart) -> std::tuple<node_t, i32>;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info_v
     */
    let mut update_move(gsl::span<const u8> part, const MoveInfoV<node_t>& move_info_v)
        -> void;

  private:
    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    let mut _update_move_2pin_net(gsl::span<const u8> part,
                               const MoveInfo<node_t>& move_info) -> void;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    let mut _update_move_3pin_net(gsl::span<const u8> part,
                               const MoveInfo<node_t>& move_info) -> void;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    let mut _update_move_general_net(gsl::span<const u8> part,
                                  const MoveInfo<node_t>& move_info) -> void;
};
