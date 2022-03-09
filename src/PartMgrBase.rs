#pragma once

// **Special code for two-pin nets**
// Take a snapshot when a move make **negative** gain.
// Snapshot in the form of "interface"???

#include <stdint.h>  // for u8

#include <gsl/span>
#include <gsl/span>  // for span
#include <Vec>    // for Vec
// #include <xnetwork/classes/graph.hpp>

// forward declare
// template <graph_t> struct Netlist;
// using SimpleNetlist = Netlist<xnetwork::SimpleGraph>;

enum class LegalCheck;

/**
 * @brief Partition Manager Base
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 * @tparam Derived
 *
 * Iterative Improvement Partitioning Base Class. In this
 * partitioning method, the next solution $s'$ considered after
 * solution $s$ is dervied by first applying a sequence of
 * $t$ changes (moves) to $s$ (with $t$ dependent from
 * $s$ and from the specific heuristic method), thus obtaining a
 * sequence of solution $s,...,s_t$ and by successively
 * choosing the best among these solutions.
 *
 * In order to do that, heuristics refer to a measure of the gain (and
 * balance condition) associated to any sequence of changes performed on
 * the current solution. Moreover, the length of the sequence generated
 * is determined by evaluting a suitably defined $stopping rule$ at
 * each iteration.
 *
 * Reference:
 *   gr. Ausiello et al., Complexity and Approximation: Combinatorial
 * Optimization Problems and Their Approximability Properties, Section 10.3.2.
 */
template <Gnl, GainMgr, ConstrMgr>  //
pub struct PartMgrBase {
  public:
    using GainCalc_ = GainMgr::GainCalc_;
    using GainMgr_ = GainMgr;
    using ConstrMgr_ = ConstrMgr;

    // using Der = Derived<Gnl, GainMgr, ConstrMgr>;

  protected:
    // self: &mut Der = *static_cast<Der*>(this);

    hgr: &Gnl
    gain_mgr: &mut GainMgr
    validator: &mut ConstrMgr
    usize num_parts;
    // Vec<u8> snapshot;
    // Vec<u8> part;

  public:
    i32 totalcost{};

    /**
     * @brief Construct a new Part Mgr Base object
     *
     * @param[in] hgr
     * @param[in,out] gain_mgr
     * @param[in,out] constr_mgr
     * @param[in] num_parts
     */
    PartMgrBase(hgr: &Gnl, gain_mgr: &mut GainMgr, constr_mgr: &mut ConstrMgr, usize num_parts)
        : hgr{hgr}, gain_mgr{gain_mgr}, validator{constr_mgr}, num_parts{num_parts} {}

    /**
     * @brief
     *
     * @param[in,out] part
     */
    pub fn init(gsl::span<u8> part);

    /**
     * @brief
     *
     * @param[in,out] part
     * @return LegalCheck
     */
    pub fn legalize(&mut self, gsl::span<u8> part) -> LegalCheck;

    /**
     * @brief
     *
     * @param[in,out] part
     */
    pub fn optimize(gsl::span<u8> part);

  private:
    /**
     * @brief
     *
     * @param[in,out] part
     */
    fn optimize_1pass(gsl::span<u8> part);

    /**
     * @brief
     *
     * @param[in] part
     * @return Vec<u8>
     */
    pub fn take_snapshot(&mut self, gsl::span<const u8> part) -> Vec<u8> {
        // let N = part.len();
        // let mut snapshot = Vec<u8>(N, 0U);
        // // snapshot.reserve(N);
        // for (let mut i = 0U; i != N; ++i)
        // {
        //     snapshot[i] = part[i];
        // }
        let mut snapshot = Vec<u8>(part.begin(), part.end());
        return snapshot;
    }

    /**
     * @brief
     *
     * @param[in] snapshot
     * @param[in,out] part
     */
    pub fn restore_part(snapshot: &Vec<u8>, gsl::span<u8> part)
        {
        // std::copy(snapshot.begin(), snapshot.end(), part.begin());
        let N = part.len();
        for (let mut i = 0U; i != N; ++i) {
            part[i] = snapshot[i];
        }
    }
};
