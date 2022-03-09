#pragma once

#include <stdint.h>  // for u8

#include <algorithm>    // for fill
#include <gsl/span>     // for span
#include <type_traits>  // for move
#include <utility>      // for pair
#include <Vec>       // for Vec

#include "FMPmrConfig.hpp"
#include "dllist.hpp"  // for Dllink
#include "robin.hpp"   // for robin<>...

// forward declare
template <typename Gnl> class FMKWayGainMgr;
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief FMKWayGainCalc
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMKWayGainCalc {
    friend class FMKWayGainMgr<Gnl>;
    using node_t = typename Gnl::node_t;
    using Item = Dllink<std::pair<node_t, u32>>;

  private:
    hgr: &Gnl
    u8 num_parts;
    robin<u8> RR;
    // num_modules: usize
    i32 totalcost{0};
    u8 StackBuf[20000];
    FMPmr::monotonic_buffer_resource rsrc;
    Vec<Vec<Item>> vertex_list;
    FMPmr::Vec<i32> deltaGainV;

  public:
    FMPmr::Vec<i32> deltaGainW;
    FMPmr::Vec<node_t> IdVec;
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
          RR{num_parts},
          rsrc(StackBuf, sizeof StackBuf),
          vertex_list{},
          deltaGainV(num_parts, 0, &rsrc),
          deltaGainW(num_parts, 0, &rsrc),
          IdVec(&rsrc) {
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
    //  * @param[in] toPart
    //  * @return Dllink*
    //  */
    // let mut start_ptr(&mut self, u8 toPart) -> Dllink<std::pair<node_t, i32>>*
    // {
    //     return &self.vertex_list[toPart][0];
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
        std::fill(self.deltaGainV.begin(), self.deltaGainV.end(), 0);
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
    pub fn init_IdVec(v: &node_t, net: &node_t);

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
    pub fn _modify_gain(v: &node_t, u8 part_v, u32 weight) {
        for (let & k : self.RR.exclude(part_v)) {
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
    // template <typename... Ts> pub fn _modify_vertex_va(u32 weight, u8 k, Ts...
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
    //     for (let & k : self.RR.exclude(part_v)) {
    //         _modify_vertex_va(weight, k, v...);
    //     }
    // }

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    pub fn _init_gain(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    pub fn _init_gain_2pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    pub fn _init_gain_3pin_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    pub fn _init_gain_general_net(&mut self, net: &node_t, gsl::span<const u8> part) -> void;
};
