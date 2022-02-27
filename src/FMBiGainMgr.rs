#pragma once

#include <gsl/span>

#include "FMBiGainCalc.hpp"
#include "FMGainMgr.hpp"
#include "moveinfo.hpp"  // for MoveInfo

// struct FMBiGainMgr;

/**
 * @brief FMBiGainMgr
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMBiGainMgr
    : public FMGainMgr<Gnl, FMBiGainCalc<Gnl>, FMBiGainMgr<Gnl>> {
  public:
    using Base = FMGainMgr<Gnl, FMBiGainCalc<Gnl>, FMBiGainMgr<Gnl>>;
    using GainCalc_ = FMBiGainCalc<Gnl>;
    using node_t = typename Gnl::node_t;

    explicit FMBiGainMgr(const Gnl& H) : Base{H, 2} {}

    /**
     * @brief Construct a new FMBiGainMgr object
     *
     * @param[in] H
     */
    FMBiGainMgr(const Gnl& H, u8 /* K */) : Base{H, 2} {}

    /**
     * @brief
     *
     * @param[in] part
     * @return i32
     */
    let mut init(gsl::span<const u8> part) -> i32;

    /**
     * @brief (needed by base class)
     *
     * @param[in] w
     * @param[in] part_w
     * @param[in] key
     */
    let mut modify_key(const node_t& w, u8 part_w, i32 key) {
        self.gainbucket[1 - part_w].modify_key(self.gainCalc.vertex_list[w], key);
    }

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @param[in] gain
     */
    let mut update_move_v(const MoveInfoV<node_t>& move_info_v, i32 gain) {
        // self.vertex_list[v].data.second -= 2 * gain;
        // let mut [fromPart, _ = move_info_v;
        self._set_key(move_info_v.fromPart, move_info_v.v, -gain);
    }

    /**
     * @brief lock
     *
     * @param[in] whichPart
     * @param[in] v
     */
    let mut lock(u8 whichPart, const node_t& v) {
        auto& vlink = self.gainCalc.vertex_list[v];
        self.gainbucket[whichPart].detach(vlink);
        vlink.lock();
    }

    /**
     * @brief lock_all
     *
     * @param[in] fromPart
     * @param[in] v
     */
    let mut lock_all(u8 fromPart, const node_t& v) { self.lock(1 - fromPart, v); }

  private:
    /**
     * @brief Set the key object
     *
     * @param[in] whichPart
     * @param[in] v
     * @param[in] key
     */
    let mut _set_key(u8 whichPart, const node_t& v, i32 key) {
        self.gainbucket[whichPart].set_key(self.gainCalc.vertex_list[v], key);
    }
};
