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

    pub fn new(hgr: &Gnl) { FMBiGainMgr : Base{hgr, 2} {}

    /**
     * @brief Construct a new FMBiGainMgr object
     *
     * @param[in] hgr
     */
    FMBiGainMgr(hgr: &Gnl, u8 /* num_parts */) : Base{hgr, 2} {}

    /**
     * @brief
     *
     * @param[in] part
     * @return i32
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> i32;

    /**
     * @brief (needed by base class)
     *
     * @param[in] w
     * @param[in] part_w
     * @param[in] key
     */
    pub fn modify_key(w: &node_t, u8 part_w, i32 key) {
        self.gainbucket[1 - part_w].modify_key(self.gain_calc.vertex_list[w], key);
    }

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @param[in] gain
     */
    pub fn update_move_v(move_info_v: &MoveInfoV<node_t>, i32 gain) {
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
    pub fn lock(u8 whichPart, v: &node_t) {
        vlink: &mut auto = self.gain_calc.vertex_list[v];
        self.gainbucket[whichPart].detach(vlink);
        vlink.lock();
    }

    /**
     * @brief lock_all
     *
     * @param[in] fromPart
     * @param[in] v
     */
    pub fn lock_all(u8 fromPart, v: &node_t) { self.lock(1 - fromPart, v); }

  private:
    /**
     * @brief Set the key object
     *
     * @param[in] whichPart
     * @param[in] v
     * @param[in] key
     */
    pub fn _set_key(u8 whichPart, v: &node_t, i32 key) {
        self.gainbucket[whichPart].set_key(self.gain_calc.vertex_list[v], key);
    }
};
