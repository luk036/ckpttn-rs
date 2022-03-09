#pragma once

#include <gsl/span>

#include "FMGainMgr.hpp"
#include "FMKWayGainCalc.hpp"

// forward declare
template <Node> struct MoveInfo;
template <Node> struct MoveInfoV;

/**
 * @brief FMKWayGainMgr
 *
 * @tparam Gnl
 */
template <Gnl> class FMKWayGainMgr
    : public FMGainMgr<Gnl, FMKWayGainCalc<Gnl>, FMKWayGainMgr<Gnl>> {
  private:
    Robin<u8> rr;

  public:
    using Base = FMGainMgr<Gnl, FMKWayGainCalc<Gnl>, FMKWayGainMgr<Gnl>>;
    using GainCalc_ = FMKWayGainCalc<Gnl>;
    using node_t = Gnl::node_t;

    /**
     * @brief Construct a new FMKWayGainMgr object
     *
     * @param[in] hgr
     * @param[in] num_parts
     */
    FMKWayGainMgr(hgr: &Gnl, u8 num_parts) : Base{hgr, num_parts}, rr{num_parts} {}

    /**
     * @brief
     *
     * @param[in] part
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> i32;

    /**
     * @brief (needed by base class)
     *
     * @param[in] w
     * @param[in] part_w
     * @param[in] keys
     */
    pub fn modify_key(w: &node_t, u8 part_w, gsl::span<const i32> keys) {
        for (let mut k : self.rr.exclude(part_w)) {
            self.gainbucket[k].modify_key(self.gain_calc.vertex_list[k][w], keys[k]);
        }
    }

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @param[in] gain
     */
    pub fn update_move_v(&mut self, move_info_v: &MoveInfoV<node_t>, i32 gain) -> void;

    /**
     * @brief lock
     *
     * @param[in] whichPart
     * @param[in] v
     */
    pub fn lock(u8 whichPart, v: &node_t) {
        vlink: &mut auto = self.gain_calc.vertex_list[whichPart][v];
        self.gainbucket[whichPart].detach(vlink);
        vlink.lock();
    }

    /**
     * @brief lock_all
     *
     * @param[in] v
     */
    pub fn lock_all(u8 /*from_part*/, v: &node_t) {
        // for (let & [vlist, bckt] :
        //     views::zip(self.gain_calc.vertex_list, self.gainbucket))
        let mut bckt_it = self.gainbucket.begin();
        for vlist in self.gain_calc.vertex_list.iter_mut() {
            vlink: &mut auto = vlist[v];
            bckt_it->detach(vlink);
            vlink.lock();  // lock
            ++bckt_it;
        }
    }

  private:
    /**
     * @brief Set the key object
     *
     * @param[in] whichPart
     * @param[in] v
     * @param[in] key
     */
    fn set_key(u8 whichPart, v: &node_t, i32 key) {
        self.gainbucket[whichPart].set_key(self.gain_calc.vertex_list[whichPart][v], key);
    }
};
