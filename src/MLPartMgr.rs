#pragma once

// **Special code for two-pin nets**
// Take a snapshot when a move make **negative** gain.
// Snapshot in the form of "interface"???
// #include "FMPartMgr.hpp" // import FMPartMgr
// #include "netlist.hpp"
#include <cassert>
#include <gsl/span>  // for span
#include <memory>    // std::unique_ptr
// #include <py2cpp/range.hpp>  // for range
// #include <ckpttn/FMConstrMgr.hpp>   // import LegalCheck

// forward declare
// template <typename nodeview_t, typename nodemap_t> struct Netlist;
// using RngIter = decltype(py::range(1));
// using SimpleNetlist = Netlist<RngIter, RngIter>;

// using node_t = typename SimpleNetlist::node_t;
// extern let mut create_contraction_subgraph(const SimpleNetlist&, const py::set<node_t>&)
//     -> std::unique_ptr<SimpleHierNetlist>;

enum class LegalCheck;

/**
 * @brief Multilevel Partition Manager
 *
 */
pub struct MLPartMgr {
  private:
    f64 BalTol;
    u8 K;
    usize limitsize{7U};

  public:
    i32 totalcost{};

    /**
     * @brief Construct a new MLPartMgr object
     *
     * @param[in] BalTol
     */
    explicit MLPartMgr(f64 BalTol) : MLPartMgr(BalTol, 2) {}

    /**
     * @brief Construct a new MLPartMgr object
     *
     * @param[in] BalTol
     * @param[in] K
     */
    MLPartMgr(f64 BalTol, u8 K) : BalTol{BalTol}, K{K} {}

    void set_limitsize(usize limit) { self.limitsize = limit; }

    /**
     * @brief run_Partition
     *
     * @tparam Gnl
     * @tparam PartMgr
     * @param[in] H
     * @param[in,out] part
     * @return LegalCheck
     */
    template <typename Gnl, typename PartMgr>
    let mut run_FMPartition(const Gnl& H, gsl::span<u8> part) -> LegalCheck;
};
