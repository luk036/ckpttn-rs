//! Shared netlist fixtures for unit tests.
//!
//! Several module-level `#[cfg(test)] mod tests` blocks used to define their own
//! copies of the same small netlists; they now import them from here.

use crate::hypergraph::SimpleNetlist;
use petgraph::graph::NodeIndex;

/// Dwarf netlist: 7 modules (a0..a3, p1..p3), 6 nets.
pub fn create_dwarf_netlist() -> SimpleNetlist {
    let mut netlist = SimpleNetlist::new(7, 6);
    let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
    let edges: Vec<(usize, usize)> = vec![
        (4, 7),
        (0, 7),
        (1, 7),
        (0, 8),
        (2, 8),
        (3, 8),
        (1, 9),
        (2, 9),
        (3, 9),
        (2, 10),
        (5, 10),
        (3, 11),
        (6, 11),
        (0, 12),
    ];
    for (u, v) in &edges {
        netlist.add_edge(nodes[*u], nodes[*v]);
    }
    netlist.module_weight = vec![1, 3, 4, 2, 0, 0, 0];
    netlist
}

/// Four modules, two nets: net 4 joins modules 0-1, net 5 joins modules 2-3.
pub fn make_nl() -> SimpleNetlist {
    let mut netlist = SimpleNetlist::new(4, 2);
    let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
    netlist.add_edge(nodes[0], nodes[4]);
    netlist.add_edge(nodes[1], nodes[4]);
    netlist.add_edge(nodes[2], nodes[5]);
    netlist.add_edge(nodes[3], nodes[5]);
    netlist
}
