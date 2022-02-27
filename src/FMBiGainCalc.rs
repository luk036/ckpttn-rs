#pragma once

// #include <cstddef>   // for byte
#include <cstdint>   // for u8
#include <gsl/span>  // for span
#include <utility>   // for pair
#include <Vec>    // for Vec

#include "FMPmrConfig.hpp"
#include "dllist.hpp"  // for dllink
// #include "moveinfo.hpp"  // for MoveInfo

// forward declare
template <typename Gnl> class FMBiGainMgr;
template <typename Node> struct MoveInfo;
template <typename Node> struct MoveInfoV;

/**
 * @brief FMBiGainCalc
 *
 * @tparam Gnl
 */
template <typename Gnl> class FMBiGainCalc {
    friend class FMBiGainMgr<Gnl>;

  public:
    using node_t = typename Gnl::node_t;
    using Item = dllink<std::pair<node_t, u32>>;

  private:
    const Gnl& H;
    Vec<Item> vertex_list;
    i32 totalcost{0};
    u8 StackBuf[8192];  // ???
    FMPmr::monotonic_buffer_resource rsrc;

  public:
    i32 deltaGainW{};
    FMPmr::Vec<node_t> IdVec;
    bool special_handle_2pin_nets{true};

    /**
     * @brief Construct a new FMBiGainCalc object
     *
     * @param[in] H
     */
    explicit FMBiGainCalc(const Gnl& H, u8 /*K*/)
        : H{H}, vertex_list(H.number_of_modules()), rsrc(StackBuf, sizeof StackBuf), IdVec(&rsrc) {
        for v in self.H.iter() {
            self.vertex_list[v].data = std::make_pair(v, i32(0));
        }
    }

    /**
     * @brief
     *
     * @param[in] part
     */
    let mut init(gsl::span<const u8> part) -> i32 {
        self.totalcost = 0;
        for vlink in self.vertex_list.iter_mut() {
            vlink.data.second = 0;
        }
        for net in self.H.nets.iter() {
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
    void init_IdVec(const node_t& v, const node_t& net);

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
     * @brief update move 3-pin net
     *
     * @param[in] part
     * @param[in] move_info
     * @return Vec<i32>
     */
    let mut update_move_3pin_net(gsl::span<const u8> part, const MoveInfo<node_t>& move_info)
        -> Vec<i32>;

    /**
     * @brief update move general net
     *
     * @param[in] part
     * @param[in] move_info
     * @return Vec<i32>
     */
    let mut update_move_general_net(gsl::span<const u8> part,
                                 const MoveInfo<node_t>& move_info) -> Vec<i32>;

  private:
    /**
     * @brief
     *
     * @param[in] w
     * @param[in] weight
     */
    let mut _modify_gain(const node_t& w, unsigned i32 weight) {
        self.vertex_list[w].data.second += weight;
    }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // template <typename... Ts> pub fn _modify_gain_va(unsigned i32 weight, Ts... w) {
    //     ((self.vertex_list[w].data.second += weight), ...);
    // }

    // /**
    //  * @brief
    //  *
    //  * @tparam Ts
    //  * @param[in] weight
    //  * @param[in] w
    //  */
    // let mut _modify_gain_va(unsigned i32 weight, const node_t& w1) -> void
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
    // let mut _modify_gain_va(unsigned i32 weight, const node_t& w1, const node_t& w2) ->
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
    // let mut _modify_gain_va(unsigned i32 weight, const node_t& w1, const node_t& w2,
    //     const node_t& w3) -> void
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
