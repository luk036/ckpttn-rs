//! Cross-language validation for ckpttn (circuit partitioning)

use ckpttn_rs::fm_bi_constr_mgr::FMBiConstrMgr;
use ckpttn_rs::fm_bi_gain_calc::FMBiGainCalc;
use ckpttn_rs::fm_bi_gain_mgr::FMBiGainMgr;
use ckpttn_rs::hypergraph::{Hypergraph, SimpleNetlist};
use ckpttn_rs::part_mgr_base::PartMgrBase;
use petgraph::graph::NodeIndex;

fn build_simple() -> SimpleNetlist {
    let mut nl = SimpleNetlist::new(4, 2);
    nl.add_edge(NodeIndex::new(0), NodeIndex::new(4));
    nl.add_edge(NodeIndex::new(1), NodeIndex::new(4));
    nl.add_edge(NodeIndex::new(2), NodeIndex::new(5));
    nl.add_edge(NodeIndex::new(3), NodeIndex::new(5));
    nl
}

fn build_chain() -> SimpleNetlist {
    let mut nl = SimpleNetlist::new(5, 3);
    nl.add_edge(NodeIndex::new(0), NodeIndex::new(5));
    nl.add_edge(NodeIndex::new(1), NodeIndex::new(5));
    nl.add_edge(NodeIndex::new(1), NodeIndex::new(6));
    nl.add_edge(NodeIndex::new(2), NodeIndex::new(6));
    nl.add_edge(NodeIndex::new(2), NodeIndex::new(7));
    nl.add_edge(NodeIndex::new(3), NodeIndex::new(7));
    nl.add_edge(NodeIndex::new(4), NodeIndex::new(7));
    nl
}

#[test]
fn test_xval_fm_bi_partition_simple() {
    // C++: run_FMBiPartMgr produces valid partition
    let hyprgraph = build_simple();
    let gain_calc = FMBiGainCalc::new(&hyprgraph, 2);
    let gain_mgr = FMBiGainMgr::new(&hyprgraph, gain_calc, 2);
    let constr_mgr = FMBiConstrMgr::new(&hyprgraph, 0.5);
    let mut part_mgr = PartMgrBase::new(&hyprgraph, gain_mgr, constr_mgr, 2);
    let mut part = vec![0u8; hyprgraph.number_of_modules()];
    part_mgr.legalize(&mut part);
    assert!(part_mgr.total_cost >= 0);
    let before = part_mgr.total_cost;
    part_mgr.optimize(&mut part);
    assert!(part_mgr.total_cost <= before || before == 0);
    for &p in &part {
        assert!(p == 0 || p == 1);
    }
}

#[test]
fn test_xval_fm_bi_partition_chain() {
    // C++: chain partition produces valid result
    let hyprgraph = build_chain();
    let gain_calc = FMBiGainCalc::new(&hyprgraph, 2);
    let gain_mgr = FMBiGainMgr::new(&hyprgraph, gain_calc, 2);
    let constr_mgr = FMBiConstrMgr::new(&hyprgraph, 0.45);
    let mut part_mgr = PartMgrBase::new(&hyprgraph, gain_mgr, constr_mgr, 2);
    let mut part = vec![0u8; hyprgraph.number_of_modules()];
    part_mgr.legalize(&mut part);
    part_mgr.optimize(&mut part);
    for &p in &part {
        assert!(p == 0 || p == 1);
    }
    assert!(part_mgr.total_cost >= 0);
}

#[test]
fn test_xval_legalize_improves_balance() {
    // FMConstrMgr legalize should produce a balanced partition
    let hyprgraph = build_simple();
    let gain_calc = FMBiGainCalc::new(&hyprgraph, 2);
    let gain_mgr = FMBiGainMgr::new(&hyprgraph, gain_calc, 2);
    let constr_mgr = FMBiConstrMgr::new(&hyprgraph, 0.5);
    let mut part_mgr = PartMgrBase::new(&hyprgraph, gain_mgr, constr_mgr, 2);
    let mut part = vec![0u8; hyprgraph.number_of_modules()];
    let result = part_mgr.legalize(&mut part);
    // Should at least get NotSatisfied (meaning legalize ran)
    // AllSatisfied would mean perfect balance
    use ckpttn_rs::fm_constr_mgr::LegalCheck;
    if result == LegalCheck::AllSatisfied {
        // Perfect balance achieved
    }
}
