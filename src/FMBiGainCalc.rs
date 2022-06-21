#pragma once

// #include <cstddef>   // for byte
#include <cstdint>   // for u8
#include <gsl/span>  // for span
#include <utility>   // for pair
#include <Vec>    // for Vec

#include "FMPmrConfig.hpp"
#include "dllist.hpp"  // for Dllink
// #include "moveinfo.hpp"  // for MoveInfo

// forward declare
template <Gnl> class FMBiGainMgr;
template <Node> struct MoveInfo;
template <Node> struct MoveInfoV;

/**
 * @brief FMBiGainCalc
 *
 * @tparam Gnl
 */
template <Gnl> class FMBiGainCalc {
    friend class FMBiGainMgr<Gnl>;

  public:
    using node_t = Gnl::node_t;
    using Item = Dllink<(node_t, u32)>;

  private:
    hgr: &Gnl
    Vec<Item> vertex_list;
    i32 totalcost{0};
    u8 stack_buf[8192];  // TODO
    FMPmr::monotonic_buffer_resource rsrc;

  public:
    i32 delta_gain_w{};
    FMPmr::Vec<node_t> idx_vec;
    bool special_handle_2pin_nets{true};

    /**
     * @brief Construct a new FMBiGainCalc object
     *
     * @param[in] hgr
     */
    pub fn new(hgr: &Gnl, u8 /*num_parts*/) { FMBiGainCalc
        : hgr{hgr}, vertex_list(hgr.number_of_modules()), rsrc(stack_buf, sizeof stack_buf), idx_vec(&rsrc) {
        for v in self.hgr.iter() {
            self.vertex_list[v].data = std::make_pair(v, i32(0));
        }
    }

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(&mut self, gsl::span<const u8> part) -> i32 {
        self.totalcost = 0;
        for vlink in self.vertex_list.iter_mut() {
            vlink.data.second = 0;
        }
        for net in self.hgr.nets.iter() {
            self._init_gain(net, part);
        }
        return self.totalcost;
    }

    /**
     * @brief update move init
     *
     */
    let mut update_move_init() {
        // nothing to do in 2-way partitioning
    }

    /**
     * @brief
     *
     * @param v
     * @param net
     */
    pub fn init_idx_vec(v: &node_t, net: &node_t);

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return node_t
     */
    let mut update_move_2pin_net(gsl::span<move_info: &u8> part, const MoveInfo<node_t>)
        -> node_t;

    /**
     * @brief update move 3-pin net
     *
     * @param[in] part
     * @param[in] move_info
     * @return Vec<i32>
     */
    let mut update_move_3pin_net(gsl::span<move_info: &u8> part, const MoveInfo<node_t>)
        -> Vec<i32>;

    /**
     * @brief update move general net
     *
     * @param[in] part
     * @param[in] move_info
     * @return Vec<i32>
     */
    let mut update_move_general_net(gsl::span<const u8> part,
                                 move_info: &MoveInfo<node_t>) -> Vec<i32>;

  private:
    /**
     * @brief
     *
     * @param[in] w
     * @param[in] weight
     */
    let mut _modify_gain(w: &node_t, u32 weight) {
        self.vertex_list[w].data.second += weight;
    }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // template <typename... Ts> fn modify_gain_va(u32 weight, Ts... w) {
    //     ((self.vertex_list[w].data.second += weight), ...);
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // let mut _modify_gain_va(&mut self, u32 weight, w1: &node_t) -> void
    // {
    //     self.vertex_list[w1].data.second += weight;
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // let mut _modify_gain_va(u32 weight, w1: &node_t, w2: &node_t) ->
    // void
    // {
    //     self.vertex_list[w1].data.second += weight;
    //     self.vertex_list[w2].data.second += weight;
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // let mut _modify_gain_va(u32 weight, w1: &node_t, w2: &node_t,
    //     w3: &node_t) -> void
    // {
    //     self.vertex_list[w1].data.second += weight;
    //     self.vertex_list[w2].data.second += weight;
    //     self.vertex_list[w3].data.second += weight;
    // }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_2pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_3pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_general_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;
};
