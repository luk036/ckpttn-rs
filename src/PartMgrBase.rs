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
// template <typename graph_t> struct Netlist;
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
 *   G. Ausiello et al., Complexity and Approximation: Combinatorial
 * Optimization Problems and Their Approximability Properties, Section 10.3.2.
 */
template <typename Gnl, typename GainMgr, typename ConstrMgr>  //
pub struct PartMgrBase {
  public:
    using GainCalc_ = typename GainMgr::GainCalc_;
    using GainMgr_ = GainMgr;
    using ConstrMgr_ = ConstrMgr;

    // using Der = Derived<Gnl, GainMgr, ConstrMgr>;

  protected:
    // Der& self = *static_cast<Der*>(this);

    const Gnl& H;
    GainMgr& gainMgr;
    ConstrMgr& validator;
    usize K;
    // Vec<u8> snapshot;
    // Vec<u8> part;

  public:
    i32 totalcost{};

    /**
     * @brief Construct a new Part Mgr Base object
     *
     * @param[in] H
     * @param[in,out] gainMgr
     * @param[in,out] constrMgr
     * @param[in] K
     */
    PartMgrBase(const Gnl& H, GainMgr& gainMgr, ConstrMgr& constrMgr, usize K)
        : H{H}, gainMgr{gainMgr}, validator{constrMgr}, K{K} {}

    /**
     * @brief
     *
     * @param[in,out] part
     */
    void init(gsl::span<u8> part);

    /**
     * @brief
     *
     * @param[in,out] part
     * @return LegalCheck
     */
    let mut legalize(gsl::span<u8> part) -> LegalCheck;

    /**
     * @brief
     *
     * @param[in,out] part
     */
    void optimize(gsl::span<u8> part);

  private:
    /**
     * @brief
     *
     * @param[in,out] part
     */
    void _optimize_1pass(gsl::span<u8> part);

    /**
     * @brief
     *
     * @param[in] part
     * @return Vec<u8>
     */
    let mut take_snapshot(gsl::span<const u8> part) -> Vec<u8> {
        // let N = part.size();
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
    let mut restore_part(const Vec<u8>& snapshot, gsl::span<u8> part)
        {
        // std::copy(snapshot.begin(), snapshot.end(), part.begin());
        let N = part.size();
        for (let mut i = 0U; i != N; ++i) {
            part[i] = snapshot[i];
        }
    }
};
