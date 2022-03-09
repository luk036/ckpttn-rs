#pragma once

#include <stdint.h>  // for u8

#include <algorithm>    // for fill
#include <gsl/span>     // for span
#include <type_traits>  // for move
#include <utility>      // for pair
#include <Vec>       // for Vec

#include "FMPmrConfig.hpp"
#include "dllist.hpp"  // for Dllink
#include "robin.hpp"   // for Robin<>...

// forward declare
template <Gnl> class FMKWayGainMgr;
template <Node> struct MoveInfo;
template <Node> struct MoveInfoV;

/**
 * @brief FMKWayGainCalc
 *
 * @tparam Gnl
 */
template <Gnl> class FMKWayGainCalc {
    friend class FMKWayGainMgr<Gnl>;
    using node_t = Gnl::node_t;
    using Item = Dllink<std::pair<node_t, u32>>;

  private:
    hgr: &Gnl
    u8 num_parts;
    Robin<u8> rr;
    // num_modules: usize
    i32 totalcost{0};
    u8 stack_buf[20000];
    FMPmr::monotonic_buffer_resource rsrc;
    Vec<Vec<Item>> vertex_list;
    FMPmr::Vec<i32> delta_gain_v;

  public:
    FMPmr::Vec<i32> delta_gain_w;
    FMPmr::Vec<node_t> idx_vec;
    bool special_handle_2pin_nets{true};  // @TODO should be template parameter

    /**
     * @brief Construct a new FMKWayGainCalc object
     *
     * @param[in] hgr Netlist
     * @param[in] num_parts number of partitions
     */
    FMKWayGainCalc(hgr: &Gnl, u8 num_parts)
        : hgr{hgr},
          num_parts{num_parts},
          rr{num_parts},
          rsrc(stack_buf, sizeof stack_buf),
          vertex_list{},
          delta_gain_v(num_parts, 0, &rsrc),
          delta_gain_w(num_parts, 0, &rsrc),
          idx_vec(&rsrc) {
        for (let mut k = 0U; k != self.num_parts; ++k) {
            let mut vec = Vec<Item>{};
            vec.reserve(hgr.number_of_modules());
            for v in self.hgr.iter() {
                vec.emplace_back(Item(std::make_pair(v, u32(0))));
            }
            self.vertex_list.emplace_back(std::move(vec));
        }
    }

    // /**
    //  * @brief
    //  *
    //  * @param[in] to_part
    //  * @return Dllink*
    //  */
    // let mut start_ptr(&mut self, u8 to_part) -> Dllink<std::pair<node_t, i32>>*
    // {
    //     return &self.vertex_list[to_part][0];
    // }

    /**
     * @brief
     *
     * @param[in] part
     */
    pub fn init(&mut self, gsl::span<const u8> part) -> i32 {
        self.totalcost = 0;
        for vec in self.vertex_list.iter_mut() {
            for vlink in vec.iter_mut() {
                vlink.data.second = 0U;
            }
        }
        for net in self.hgr.nets.iter() {
            self._init_gain(net, part);
        }
        return self.totalcost;
    }

    /**
     * @brief
     *
     */
    pub fn update_move_init() {
        std::fill(self.delta_gain_v.begin(), self.delta_gain_v.end(), 0);
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return node_t
     */
    pub fn update_move_2pin_net(gsl::span<move_info: &u8> part, const MoveInfo<node_t>)
        -> node_t;

    /**
     * @brief
     *
     * @param[in] v
     * @param[in] net
     */
    pub fn init_idx_vec(v: &node_t, net: &node_t);

    using ret_info = Vec<Vec<i32>>;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    pub fn update_move_3pin_net(gsl::span<move_info: &u8> part, const MoveInfo<node_t>)
        -> ret_info;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    pub fn update_move_general_net(gsl::span<const u8> part,
                                 move_info: &MoveInfo<node_t>) -> ret_info;

  private:
    /**
     * @brief
     *
     * @param[in] v
     * @param[in] part_v
     * @param[in] weight
     */
    fn modify_gain(v: &node_t, u8 part_v, u32 weight) {
        for (let & k : self.rr.exclude(part_v)) {
            self.vertex_list[k][v].data.second += weight;
        }
    }

    /**
     * @brief
     *
     * @tparam Ts
     * @param[in] weight
     * @param[in] k
     * @param[in] v
     */
    // template <typename... Ts> fn modify_vertex_va(u32 weight, u8 k, Ts...
    // v)
    //     {
    //     ((self.vertex_list[k][v].data.second += weight), ...);
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] part_v
    //  * @param[in] v
    //  */
    // let mut _modify_vertex_va(u32 weight, u8 k, v1: &node_t) ->
    // void
    // {
    //     self.vertex_list[k][v1].data.second += weight;
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] part_v
    //  * @param[in] v
    //  */
    // let mut _modify_vertex_va(
    //     u32 weight, u8 k, v1: &node_t, v2: &node_t) ->
    //     void
    // {
    //     self.vertex_list[k][v1].data.second += weight;
    //     self.vertex_list[k][v2].data.second += weight;
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] part_v
    //  * @param[in] v
    //  */
    // let mut _modify_vertex_va(u32 weight, u8 k, v1: &node_t,
    //     v2: &node_t, v3: &node_t) -> void
    // {
    //     self.vertex_list[k][v1].data.second += weight;
    //     self.vertex_list[k][v2].data.second += weight;
    //     self.vertex_list[k][v3].data.second += weight;
    // }

    /**
     * @brief
     *
     * @tparam Ts
     * @param[in] weight
     * @param[in] part_v
     * @param[in] v
     */
    // template <typename... Ts>
    // let mut _modify_gain_va(u32 weight, u8 part_v, Ts... v) {
    //     for (let & k : self.rr.exclude(part_v)) {
    //         _modify_vertex_va(weight, k, v...);
    //     }
    // }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_2pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_3pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    fn init_gain_general_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;
};
