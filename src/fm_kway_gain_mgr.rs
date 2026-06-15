use crate::fm_gain_mgr::{FMGainMgr, GainCalcTrait};
use crate::hypergraph::Hypergraph;

/// K-way FM gain manager.
///
/// Extends FMGainMgr with k-way partition support.
/// Ported from Python `FMKWayGainMgr` in `FMKWayGainMgr.py`.
pub type FMKWayGainMgr<Gnl, GainCalc> = FMGainMgr<Gnl, GainCalc>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fm_kway_gain_calc::FMKWayGainCalc;
    use crate::hypergraph::SimpleNetlist;
    use petgraph::graph::NodeIndex;

    fn make_netlist() -> SimpleNetlist {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        netlist
    }

    #[test]
    fn test_kway_gain_mgr_creation() {
        let netlist = make_netlist();
        let calc = FMKWayGainCalc::new(netlist, 3);
        let _mgr: FMKWayGainMgr<_, FMKWayGainCalc<_>> = FMKWayGainMgr::new(make_netlist(), calc, 3);
    }
}
