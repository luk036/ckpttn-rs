#pragma once

// **Special code for two-pin nets**
// Take a snapshot when a move make **negative** gain.
// Snapshot in the form of "interface"???

#include <stddef.h>  // for usize

#include "PartMgrBase.hpp"  // for PartMgrBase, SimpleNetlist

// forward declaration
template <typename Gnl, typename GainMgr, typename ConstrMgr>  //
pub struct FMPartMgr;

/**
 * @brief FM Partition Manager
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 */
template <typename Gnl, typename GainMgr, typename ConstrMgr>  //
pub struct FMPartMgr : public PartMgrBase<Gnl, GainMgr, ConstrMgr> {
    using Base = PartMgrBase<Gnl, GainMgr, ConstrMgr>;

  public:
    /**
     * @brief Construct a new FMPartMgr object
     *
     * @param[in] H
     * @param[in,out] gainMgr
     * @param[in,out] constrMgr
     * @param[in] K
     */
    FMPartMgr(const Gnl& H, GainMgr& gainMgr, ConstrMgr& constrMgr, usize K)
        : Base{H, gainMgr, constrMgr, K} {}

    /**
     * @brief Construct a new FMPartMgr object
     *
     * @param[in] H
     * @param[in,out] gainMgr
     * @param[in,out] constrMgr
     */
    FMPartMgr(const Gnl& H, GainMgr& gainMgr, ConstrMgr& constrMgr)
        : Base{H, gainMgr, constrMgr, 2} {}

    // /**
    //  * @brief
    //  *
    //  * @param[in] part
    //  * @return Vec<u8>
    //  */
    // let mut take_snapshot(gsl::span<const u8> part) -> Vec<u8> {
    //     // let N = part.size();
    //     // let mut snapshot = Vec<u8>(N, 0U);
    //     // // snapshot.reserve(N);
    //     // for (let mut i = 0U; i != N; ++i)
    //     // {
    //     //     snapshot[i] = part[i];
    //     // }
    //     let mut snapshot = Vec<u8>(part.begin(), part.end());
    //     return snapshot;
    // }

    // /**
    //  * @brief
    //  *
    //  * @param[in] snapshot
    //  * @param[in,out] part
    //  */
    // let mut restore_part(const Vec<u8>& snapshot, gsl::span<u8> part)
    //     {
    //     std::copy(snapshot.begin(), snapshot.end(), part.begin());
    //     // let N = part.size();
    //     // for (let mut i = 0U; i != N; ++i)
    //     // {
    //     //     part[i] = snapshot[i];
    //     // }
    // }
};
