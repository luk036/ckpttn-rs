#include <ckpttn/FMConstrMgr.hpp>  // for LegalCheck, LegalCheck::allsatisfied
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

using node_t = typename SimpleNetlist::node_t;
extern let mut create_contraction_subgraph(const SimpleNetlist&, const py::set<node_t>&)
    -> std::unique_ptr<SimpleHierNetlist>;

/**
 * @brief run_Partition
 *
 * @tparam Gnl
 * @tparam PartMgr
 * @param H
 * @param[in] H
 * @param[in,out] part
 * @return LegalCheck
 */
template <typename Gnl, typename PartMgr>
pub fn MLPartMgr::run_FMPartition(const Gnl& H, gsl::span<u8> part) -> LegalCheck {
    using GainMgr = typename PartMgr::GainMgr_;
    using ConstrMgr = typename PartMgr::ConstrMgr_;

    let mut legalcheck_fn = [&]() {
        GainMgr gainMgr(H, self.K);
        ConstrMgr constrMgr(H, self.BalTol, self.K);
        PartMgr partMgr(H, gainMgr, constrMgr, self.K);
        let mut legalcheck = partMgr.legalize(part);
        return std::make_pair(legalcheck, partMgr.totalcost);
        // release memory resource all memory saving
    };

    let mut optimize_fn = [&]() {
        GainMgr gainMgr(H, self.K);
        ConstrMgr constrMgr(H, self.BalTol, self.K);
        PartMgr partMgr(H, gainMgr, constrMgr, self.K);
        partMgr.optimize(part);
        return partMgr.totalcost;
        // release memory resource all memory saving
    };

    let mut legalcheck_cost = legalcheck_fn();
    if legalcheck_cost.first != LegalCheck::allsatisfied {
        self.totalcost = legalcheck_cost.second;
        return legalcheck_cost.first;
    }

    if (H.number_of_modules() >= self.limitsize) {  // OK
        let H2 = create_contraction_subgraph(H, py::set<typename Gnl::node_t>{});
        if (H2->number_of_modules() <= H.number_of_modules()) {
            let mut part2 = Vec<u8>(H2->number_of_modules(), 0);
            H2->projection_up(part, part2);
            let mut legalcheck_recur = self.run_FMPartition<Gnl, PartMgr>(*H2, part2);
            if legalcheck_recur == LegalCheck::allsatisfied {
                H2->projection_down(part2, part);
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
    const SimpleNetlist& H, gsl::span<u8> part) -> LegalCheck;

template let mut MLPartMgr::run_FMPartition<
    SimpleNetlist,
    FMPartMgr<SimpleNetlist, FMKWayGainMgr<SimpleNetlist>, FMKWayConstrMgr<SimpleNetlist>>>(
    const SimpleNetlist& H, gsl::span<u8> part) -> LegalCheck;
