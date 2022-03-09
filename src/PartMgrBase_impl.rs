#include <cassert>                 // for assert
#include <ckpttn/FMConstrMgr.hpp>  // for LegalCheck, LegalCheck::notsat...
#include <ckpttn/PartMgrBase.hpp>  // for PartMgrBase, part, SimpleNetlist
#include <ckpttn/moveinfo.hpp>     // for MoveInfoV
#include <cstdint>                 // for u8
#include <gsl/span>                // for span
#include <py2cpp/range.hpp>        // for _iterator
#include <py2cpp/set.hpp>          // for set
#include <tuple>                   // for tuple_element<>::type
#include <tuple>                   // for get
#include <Vec>                  // for Vec

// using node_t = SimpleNetlist::node_t;
// using namespace std;

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 * @param[in] part
 */
template <Gnl, GainMgr, ConstrMgr>  //
void PartMgrBase<Gnl, GainMgr, ConstrMgr>::init(gsl::span<u8> part) {
    self.totalcost = self.gain_mgr.init(part);
    self.validator.init(part);
}

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 * @param[in] part
 * @return LegalCheck
 */
template <Gnl, GainMgr, ConstrMgr>  //
pub fn PartMgrBase<Gnl, GainMgr, ConstrMgr>::legalize(&mut self, gsl::span<u8> part) -> LegalCheck {
    self.init(part);

    // Zero-weighted modules does not contribute legalization
    for v in self.hgr.iter() {
        if (self.hgr.get_module_weight(v) != 0U) {
            continue;
        }
        if !self.hgr.module_fixed.contains(v) {
            continue;
        }
        self.gain_mgr.lock_all(part[v], v);
    }

    let mut legalcheck = LegalCheck::NotStatisfied;
    while legalcheck != LegalCheck::AllStatisfied {
        let to_part = self.validator.select_togo();
        if self.gain_mgr.is_empty_togo(to_part) {
            break;
        }
        let rslt = self.gain_mgr.select_togo(to_part);
        let mut v = std::get<0>(rslt);
        let mut gainmax = std::get<1>(rslt);
        let from_part = part[v];
        // assert!(v == v);
        assert!(from_part != to_part);
        let move_info_v = MoveInfoV<Gnl::node_t>{v, from_part, to_part};
        // Check if the move of v can NotStatisfied, makebetter, or satisfied
        legalcheck = self.validator.check_legal(move_info_v);
        if legalcheck == LegalCheck::NotStatisfied {  // NotStatisfied
            continue;
        }
        // Update v and its neigbours (even they are in waitinglist);
        // Put neigbours to bucket
        self.gain_mgr.update_move(part, move_info_v);
        self.gain_mgr.update_move_v(move_info_v, gainmax);
        self.validator.update_move(move_info_v);
        part[v] = to_part;
        // totalgain += gainmax;
        self.totalcost -= gainmax;
        assert!(self.totalcost >= 0);
    }
    return legalcheck;
}

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 * @param[in] part
 */
template <Gnl, GainMgr, ConstrMgr>  //
void PartMgrBase<Gnl, GainMgr, ConstrMgr>::_optimize_1pass(gsl::span<u8> part) {
    // using SS_t = decltype(self.take_snapshot(part));
    using SS_t = Vec<u8>;

    let mut snapshot = SS_t{};
    let mut totalgain = 0;
    let mut deferredsnapshot = false;
    let mut besttotalgain = 0;

    while (!self.gain_mgr.is_empty()) {
        // Take the gainmax with v from gainbucket
        // let mut [move_info_v, gainmax] = self.gain_mgr.select(part);
        let mut result = self.gain_mgr.select(part);
        let mut move_info_v = std::get<0>(result);
        let mut gainmax = std::get<1>(result);

        // Check if the move of v can satisfied or NotStatisfied
        let satisfiedOK = self.validator.check_constraints(move_info_v);
        if !satisfiedOK {
            continue;
        }
        if gainmax < 0 {
            // become down turn
            if !deferredsnapshot || totalgain > besttotalgain {
                // Take a snapshot before move
                // snapshot = part;
                snapshot = self.take_snapshot(part);
                besttotalgain = totalgain;
            }
            deferredsnapshot = true;
        } else if totalgain + gainmax >= besttotalgain {
            besttotalgain = totalgain + gainmax;
            deferredsnapshot = false;
        }
        // Update v and its neigbours (even they are in waitinglist);
        // Put neigbours to bucket
        // let & [v, _, to_part] = move_info_v;
        self.gain_mgr.lock(move_info_v.to_part, move_info_v.v);
        self.gain_mgr.update_move(part, move_info_v);
        self.gain_mgr.update_move_v(move_info_v, gainmax);
        self.validator.update_move(move_info_v);
        totalgain += gainmax;
        part[move_info_v.v] = move_info_v.to_part;
    }
    if deferredsnapshot {
        // restore the previous best solution
        // part = snapshot;
        self.restore_part(snapshot, part);
        totalgain = besttotalgain;
    }
    self.totalcost -= totalgain;
}

/**
 * @brief
 *
 * @tparam Gnl
 * @tparam GainMgr
 * @tparam ConstrMgr
 * @tparam Derived
 * @param[in] part
 */
template <Gnl, GainMgr, ConstrMgr>  //
void PartMgrBase<Gnl, GainMgr, ConstrMgr>::optimize(gsl::span<u8> part) {
    // self.init(part);
    // let mut totalcostafter = self.totalcost;
    while true {
        self.init(part);
        let mut totalcostbefore = self.totalcost;
        // assert!(totalcostafter == totalcostbefore);
        self._optimize_1pass(part);
        assert!(self.totalcost <= totalcostbefore);
        if self.totalcost == totalcostbefore {
            break;
        }
        // totalcostafter = self.totalcost;
    }
}

#include <ckpttn/FMKWayConstrMgr.hpp>  // for FMKWayConstrMgr
#include <ckpttn/FMKWayGainMgr.hpp>    // for FMKWayGainMgr
#include <ckpttn/FMPartMgr.hpp>        // for FMPartMgr
#include <ckpttn/netlist.hpp>          // for SimpleNetlist, Netlist
#include <xnetwork/classes/graph.hpp>

template class PartMgrBase<SimpleNetlist, FMKWayGainMgr<SimpleNetlist>,
                           FMKWayConstrMgr<SimpleNetlist>>;

#include <ckpttn/FMBiConstrMgr.hpp>  // for FMBiConstrMgr
#include <ckpttn/FMBiGainMgr.hpp>    // for FMBiGainMgr

template class PartMgrBase<SimpleNetlist, FMBiGainMgr<SimpleNetlist>, FMBiConstrMgr<SimpleNetlist>>;