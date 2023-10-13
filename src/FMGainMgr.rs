#pragma once

#include <algorithm>  // for all_of
#include <cinttypes>  // for u8, u32
#include <gsl/span>   // for span
#include <tuple>      // for tuple
#include <utility>    // for pair
#include <Vec>     // for Vec<>::const_iterator, Vec

#include "BPQueue.hpp"  // for BPQueue
#include "dllist.hpp"   // for Dllink

template <Node> struct MoveInfo;
template <Node> struct MoveInfoV;

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainCalc
 * @tparam Derived
 */
template <Gnl, GainCalc, class Derived> class FMGainMgr {
    self: &mut Derived = *static_cast<Derived*>(this);
    using node_t = Gnl::node_t;
    // friend Derived;
    using Item = Dllink<(node_t, u32)>;

  protected:
    Item waitinglist{std::make_pair(node_t{}, u32(0))};
    hyprgraph: &Gnl
    Vec<BPQueue<node_t>> gainbucket;
    // pmax: usize
    u8 num_parts;

  public:
    GainCalc gain_calc;

    // i32 totalcost;

    // FMGainMgr(FMGainMgr&&) = default;

    /**
     * @brief Construct a new FMGainMgr object
     *
     * @param[in] hyprgraph
     * @param[in] num_parts
     */
    FMGainMgr(hyprgraph: &Gnl, u8 num_parts);

    /**
     * @brief
     *
     * @param[in] part
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> i32;

    /**
     * @brief
     *
     * @param[in] to_part
     * @return true
     * @return false
     */
    pub fn is_empty_togo(&self, u8 to_part) -> bool {
        return self.gainbucket[to_part].is_empty();
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
    pub fn select(&mut self, gsl::span<const u8> part) -> std::tuple<MoveInfoV<node_t>, i32>;

    /**
     * @brief
     *
     * @param[in] to_part
     * @return std::tuple<node_t, i32>
     */
    pub fn select_togo(&mut self, u8 to_part) -> std::tuple<node_t, i32>;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info_v
     */
    pub fn update_move(gsl::span<move_info_v: &u8> part, const MoveInfoV<node_t>)
        -> void;

  private:
    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    fn update_move_2pin_net(gsl::span<const u8> part,
                               move_info: &MoveInfo<node_t>) -> void;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    fn update_move_3pin_net(gsl::span<const u8> part,
                               move_info: &MoveInfo<node_t>) -> void;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     */
    fn update_move_general_net(gsl::span<const u8> part,
                                  move_info: &MoveInfo<node_t>) -> void;
};
