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
// template <nodeview_t, nodemap_t> struct Netlist;
// using RngIter = decltype(py::range(1));
// using SimpleNetlist = Netlist<RngIter, RngIter>;

// using node_t = SimpleNetlist::node_t;
// extern let mut create_contraction_subgraph(const SimpleNetlist&, const py::set<node_t>&)
//     -> std::unique_ptr<SimpleHierNetlist>;

enum class LegalCheck;

/**
 * @brief Multilevel Partition Manager
 *
 */
pub struct MLPartMgr {
  private:
    f64 bal_tol;
    u8 num_parts;
    limitsize: usize{7U};

  public:
    i32 totalcost{};

    /**
     * @brief Construct a new MLPartMgr object
     *
     * @param[in] bal_tol
     */
    pub fn new(f64 bal_tol) { MLPartMgr : MLPartMgr(bal_tol, 2) {}

    /**
     * @brief Construct a new MLPartMgr object
     *
     * @param[in] bal_tol
     * @param[in] num_parts
     */
    MLPartMgr(f64 bal_tol, u8 num_parts) : bal_tol{bal_tol}, num_parts{num_parts} {}

    pub fn set_limitsize(limit: usize) { self.limitsize = limit; }

    /**
     * @brief run_Partition
     *
     * @tparam Gnl
     * @tparam PartMgr
     * @param[in] hyprgraph
     * @param[in,out] part
     * @return LegalCheck
     */
    template <Gnl, PartMgr>
    pub fn run_FMPartition(&mut self, hyprgraph: &Gnl, gsl::span<u8> part) -> LegalCheck;
};
