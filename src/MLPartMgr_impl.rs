#include <ckpttn/FMConstrMgr.hpp>  // for LegalCheck, LegalCheck::AllStatisfied
#include <ckpttn/MLPartMgr.hpp>    // for MLPartMgr
#include <cstdint>                 // for u8
#include <gsl/span>                // for span
#include <memory>                  // for unique_ptr
#include <py2cpp/set.hpp>          // for set
#include <utility>                 // for pair
#include <Vec>                  // for Vec

#include "ckpttn/HierNetlist.hpp"  // for HierNetlist, SimpleHierNetlist
#include "ckpttn/netlist.hpp"      // for SimpleNetlist
// #include <iostream>

using node_t = SimpleNetlist::node_t;
extern let mut create_contraction_subgraph(const SimpleNetlist&, const py::set<node_t>&)
    -> std::unique_ptr<SimpleHierNetlist>;

/**
 * @brief run_Partition
 *
 * @tparam Gnl
 * @tparam PartMgr
 * @param hyprgraph
 * @param[in] hyprgraph
 * @param[in,out] part
 * @return LegalCheck
 */
template <Gnl, PartMgr>
pub fn MLPartMgr::run_FMPartition(&mut self, hyprgraph: &Gnl, gsl::span<u8> part) -> LegalCheck {
    using GainMgr = PartMgr::GainMgr_;
    using ConstrMgr = PartMgr::ConstrMgr_;

    let mut legalcheck_fn = [&]() {
        GainMgr gain_mgr(hyprgraph, self.num_parts);
        ConstrMgr constr_mgr(hyprgraph, self.bal_tol, self.num_parts);
        PartMgr part_mgr(hyprgraph, gain_mgr, constr_mgr, self.num_parts);
        let mut legalcheck = part_mgr.legalize(part);
        return std::make_pair(legalcheck, part_mgr.totalcost);
        // release memory resource all memory saving
    };

    let mut optimize_fn = [&]() {
        GainMgr gain_mgr(hyprgraph, self.num_parts);
        ConstrMgr constr_mgr(hyprgraph, self.bal_tol, self.num_parts);
        PartMgr part_mgr(hyprgraph, gain_mgr, constr_mgr, self.num_parts);
        part_mgr.optimize(part);
        return part_mgr.totalcost;
        // release memory resource all memory saving
    };

    let mut legalcheck_cost = legalcheck_fn();
    if legalcheck_cost.first != LegalCheck::AllStatisfied {
        self.totalcost = legalcheck_cost.second;
        return legalcheck_cost.first;
    }

    if (hyprgraph.number_of_modules() >= self.limitsize) {  // OK
        let hgr2 = create_contraction_subgraph(hyprgraph, py::set<Gnl::node_t>{});
        if (hgr2->number_of_modules() <= hyprgraph.number_of_modules()) {
            let mut part2 = Vec<u8>(hgr2->number_of_modules(), 0);
            hgr2->projection_up(part, part2);
            let mut legalcheck_recur = self.run_FMPartition<Gnl, PartMgr>(*hgr2, part2);
            if legalcheck_recur == LegalCheck::AllStatisfied {
                hgr2->projection_down(part2, part);
            }
        }
    }

    self.totalcost = optimize_fn();
    return legalcheck_cost.first;
}

#include <ckpttn/FMBiConstrMgr.hpp>    // for FMBiConstrMgr
#include <ckpttn/FMBiGainMgr.hpp>      // for FMBiGainMgr
#include <ckpttn/FMKWayConstrMgr.hpp>  // for FMKWayConstrMgr
#include <ckpttn/FMKWayGainMgr.hpp>    // for FMKWayGainMgr
#include <ckpttn/FMPartMgr.hpp>        // for FMPartMgr

template let mut MLPartMgr::run_FMPartition<
    SimpleNetlist,
    FMPartMgr<SimpleNetlist, FMBiGainMgr<SimpleNetlist>, FMBiConstrMgr<SimpleNetlist>>>(
    hyprgraph: &SimpleNetlist, gsl::span<u8> part) -> LegalCheck;

template let mut MLPartMgr::run_FMPartition<
    SimpleNetlist,
    FMPartMgr<SimpleNetlist, FMKWayGainMgr<SimpleNetlist>, FMKWayConstrMgr<SimpleNetlist>>>(
    hyprgraph: &SimpleNetlist, gsl::span<u8> part) -> LegalCheck;
