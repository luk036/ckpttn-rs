#pragma once

// **Special code for two-pin nets**
// Take a snapshot when a move make **negative** gain.
// Snapshot in the form of "interface"???

#include <stddef.h>  // for usize

#include "PartMgrBase.hpp"  // for PartMgrBase, SimpleNetlist

// forward declaration
template <Gnl, GainMgr, ConstrMgr>  //
pub struct FMPartMgr;

/**
 * @brief FM Partition Manager
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 */
template <Gnl, GainMgr, ConstrMgr>  //
pub struct FMPartMgr : public PartMgrBase<Gnl, GainMgr, ConstrMgr> {
    using Base = PartMgrBase<Gnl, GainMgr, ConstrMgr>;

  public:
    /**
     * @brief Construct a new FMPartMgr object
     *
     * @param[in] hgr
     * @param[in,out] gain_mgr
     * @param[in,out] constr_mgr
     * @param[in] num_parts
     */
    FMPartMgr(hgr: &Gnl, gain_mgr: &mut GainMgr, constr_mgr: &mut ConstrMgr, usize num_parts)
        : Base{hgr, gain_mgr, constr_mgr, num_parts} {}

    /**
     * @brief Construct a new FMPartMgr object
     *
     * @param[in] hgr
     * @param[in,out] gain_mgr
     * @param[in,out] constr_mgr
     */
    FMPartMgr(hgr: &Gnl, gain_mgr: &mut GainMgr, constr_mgr: &mut ConstrMgr)
        : Base{hgr, gain_mgr, constr_mgr, 2} {}

    // /**
    //  * @brief
    //  *
    //  * @param[in] part
    //  * @return Vec<u8>
    //  */
    // let mut take_snapshot(&mut self, gsl::span<const u8> part) -> Vec<u8> {
    //     // let N = part.len();
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
    // let mut restore_part(snapshot: &Vec<u8>, gsl::span<u8> part)
    //     {
    //     std::copy(snapshot.begin(), snapshot.end(), part.begin());
    //     // let N = part.len();
    //     // for (let mut i = 0U; i != N; ++i)
    //     // {
    //     //     part[i] = snapshot[i];
    //     // }
    // }
};
