#pragma once

#include <stdint.h>  // for u8

#include <algorithm>    // for fill
#include <gsl/span>     // for span
#include <type_traits>  // for move
#include <utility>      // for pair
#include <Vec>       // for Vec

#include "FMPmrConfig.hpp"
#include "dllist.hpp"  // for dllink
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
    using Item = dllink<std::pair<node_t, u32>>;

  private:
    const Gnl& H;
    u8 K;
    robin<u8> RR;
    // usize num_modules;
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
     * @param[in] H Netlist
     * @param[in] K number of partitions
     */
    FMKWayGainCalc(const Gnl& H, u8 K)
        : H{H},
          K{K},
          RR{K},
          rsrc(StackBuf, sizeof StackBuf),
          vertex_list{},
          deltaGainV(K, 0, &rsrc),
          deltaGainW(K, 0, &rsrc),
          IdVec(&rsrc) {
        for (let mut k = 0U; k != self.K; ++k) {
            let mut vec = Vec<Item>{};
            vec.reserve(H.number_of_modules());
            for v in self.H.iter() {
                vec.emplace_back(Item(std::make_pair(v, u32(0))));
            }
            self.vertex_list.emplace_back(std::move(vec));
        }
    }

    // /**
    //  * @brief
    //  *
    //  * @param[in] toPart
    //  * @return dllink*
    //  */
    // let mut start_ptr(u8 toPart) -> dllink<std::pair<node_t, i32>>*
    // {
    //     return &self.vertex_list[toPart][0];
    // }

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) -> i32 {
        self.totalcost = 0;
        for vec in self.vertex_list.iter_mut() {
            for vlink in vec.iter_mut() {
                vlink.data.second = 0U;
            }
        }
        for net in self.H.nets.iter() {
            self._init_gain(net, part);
        }
        return self.totalcost;
    }

    /**
     * @brief
     *
     */
    let mut update_move_init() {
        std::fill(self.deltaGainV.begin(), self.deltaGainV.end(), 0);
    }

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return node_t
     */
    let mut update_move_2pin_net(gsl::span<const u8> part, const MoveInfo<node_t>& move_info)
        -> node_t;

    /**
     * @brief
     *
     * @param[in] v
     * @param[in] net
     */
    void init_IdVec(const node_t& v, const node_t& net);

    using ret_info = Vec<Vec<i32>>;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    let mut update_move_3pin_net(gsl::span<const u8> part, const MoveInfo<node_t>& move_info)
        -> ret_info;

    /**
     * @brief
     *
     * @param[in] part
     * @param[in] move_info
     * @return ret_info
     */
    let mut update_move_general_net(gsl::span<const u8> part,
                                 const MoveInfo<node_t>& move_info) -> ret_info;

  private:
    /**
     * @brief
     *
     * @param[in] v
     * @param[in] part_v
     * @param[in] weight
     */
    let mut _modify_gain(const node_t& v, u8 part_v, unsigned i32 weight) {
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
    // template <typename... Ts> pub fn _modify_vertex_va(unsigned i32 weight, u8 k, Ts...
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
    // let mut _modify_vertex_va(unsigned i32 weight, u8 k, const node_t& v1) ->
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
    //     unsigned i32 weight, u8 k, const node_t& v1, const node_t& v2) ->
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
    // let mut _modify_vertex_va(unsigned i32 weight, u8 k, const node_t& v1,
    //     const node_t& v2, const node_t& v3) -> void
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
    // let mut _modify_gain_va(unsigned i32 weight, u8 part_v, Ts... v) {
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
    let mut _init_gain(const node_t& net, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_2pin_net(const node_t& net, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_3pin_net(const node_t& net, gsl::span<const u8> part) -> void;

    /**
     * @brief
     *
     * @param[in] net
     * @param[in] part
     */
    let mut _init_gain_general_net(const node_t& net, gsl::span<const u8> part) -> void;
};
