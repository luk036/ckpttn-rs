#pragma once

#include <gsl/span>

#include "FMGainMgr.hpp"
#include "FMKWayGainCalc.hpp"

// forward declare
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief FMKWayGainMgr
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMKWayGainMgr
    : public FMGainMgr<Gnl, FMKWayGainCalc<Gnl>, FMKWayGainMgr<Gnl>> {
  private:
    robin<u8> RR;

  public:
    using Base = FMGainMgr<Gnl, FMKWayGainCalc<Gnl>, FMKWayGainMgr<Gnl>>;
    using GainCalc_ = FMKWayGainCalc<Gnl>;
    using node_t = typename Gnl::node_t;

    /**
     * @brief Construct a new FMKWayGainMgr object
     *
     * @param[in] H
     * @param[in] K
     */
    FMKWayGainMgr(const Gnl& H, u8 K) : Base{H, K}, RR{K} {}

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) -> i32;

    /**
     * @brief (needed by base class)
     *
     * @param[in] w
     * @param[in] part_w
     * @param[in] keys
     */
    let mut modify_key(const node_t& w, u8 part_w, gsl::span<const i32> keys) {
        for (let mut k : self.RR.exclude(part_w)) {
            self.gainbucket[k].modify_key(self.gainCalc.vertex_list[k][w], keys[k]);
        }
    }

    /**
     * @brief
     *
     * @param[in] move_info_v
     * @param[in] gain
     */
    let mut update_move_v(const MoveInfoV<node_t>& move_info_v, i32 gain) -> void;

    /**
     * @brief lock
     *
     * @param[in] whichPart
     * @param[in] v
     */
    let mut lock(u8 whichPart, const node_t& v) {
        auto& vlink = self.gainCalc.vertex_list[whichPart][v];
        self.gainbucket[whichPart].detach(vlink);
        vlink.lock();
    }

    /**
     * @brief lock_all
     *
     * @param[in] v
     */
    let mut lock_all(u8 /*fromPart*/, const node_t& v) {
        // for (let & [vlist, bckt] :
        //     views::zip(self.gainCalc.vertex_list, self.gainbucket))
        let mut bckt_it = self.gainbucket.begin();
        for vlist in self.gainCalc.vertex_list.iter_mut() {
            auto& vlink = vlist[v];
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
    let mut _set_key(u8 whichPart, const node_t& v, i32 key) {
        self.gainbucket[whichPart].set_key(self.gainCalc.vertex_list[whichPart][v], key);
    }
};
