use std::collections::{HashMap, HashSet};

use petgraph::graph::NodeIndex;

use crate::hier_netlist::HierNetlist;
use crate::hypergraph::Hypergraph;

/// Contract a subgraph by clustering connected modules.
///
/// This is the main entry point for the contraction/clustering algorithm.
/// It finds a matching, creates clusters, builds a new hierarchical netlist,
/// and purges duplicate nets.
/// Ported from Python `min_cover.py`.
pub fn contract_subgraph(
    hyprgraph: &impl Hypergraph<Node = NodeIndex>,
    module_weight: &[u32],
    forbid: &HashSet<NodeIndex>,
) -> (HierNetlist, Vec<u32>) {
    // Step 1: Compute cluster weights
    let cluster_weight: HashMap<NodeIndex, u32> = hyprgraph
        .nets()
        .map(|net| {
            let w: u32 = hyprgraph
                .neighbors(net)
                .map(|v| {
                    let idx = v.index();
                    if idx < module_weight.len() {
                        module_weight[idx]
                    } else {
                        1
                    }
                })
                .sum();
            (net, w)
        })
        .collect();

    // Step 2: Find matching (simple greedy approach)
    let nets: Vec<NodeIndex> = hyprgraph.nets().collect();

    // Greedy matching: sort nets by cluster weight descending, take non-overlapping
    let mut sorted_nets: Vec<(NodeIndex, u32)> = nets
        .iter()
        .map(|&n| (n, *cluster_weight.get(&n).unwrap_or(&0)))
        .collect();
    sorted_nets.sort_by_key(|b| std::cmp::Reverse(b.1));

    let mut matched_nets_set: HashSet<NodeIndex> = HashSet::new();
    let mut covered_modules: HashSet<usize> = HashSet::new();

    for &(net, _) in &sorted_nets {
        if forbid.contains(&net) {
            continue;
        }
        let module_idxs: Vec<usize> = hyprgraph.neighbors(net).map(|v| v.index()).collect();
        let any_covered = module_idxs.iter().any(|idx| covered_modules.contains(idx));
        if !any_covered && module_idxs.len() >= 2 {
            matched_nets_set.insert(net);
            for &idx in &module_idxs {
                covered_modules.insert(idx);
            }
        }
    }

    // Step 3: Separate clusters and remaining nets
    let mut clusters: Vec<NodeIndex> = Vec::new();
    let mut remaining_nets: Vec<NodeIndex> = Vec::new();
    for &net in hyprgraph.nets().collect::<Vec<_>>().iter() {
        if matched_nets_set.contains(&net) {
            clusters.push(net);
        } else {
            remaining_nets.push(net);
        }
    }

    // Step 4: Build cell list (modules not in any cluster)
    let all_modules: Vec<NodeIndex> = hyprgraph.modules().collect();
    let cell_list: Vec<NodeIndex> = all_modules
        .iter()
        .filter(|&&v| !covered_modules.contains(&v.index()))
        .copied()
        .collect();

    let num_cells = cell_list.len();
    let num_clusters = clusters.len();
    let num_modules = num_cells + num_clusters;

    // Step 5: Construct new graph
    let gr2 = construct_graph(hyprgraph, &remaining_nets, &cell_list, &clusters);

    // Step 6: Build HierNetlist
    let num_remaining_nets = remaining_nets.len();
    let mut hgr2 = HierNetlist::new(num_modules, num_remaining_nets);

    // Copy edges from constructed graph
    for edge in gr2.raw_edges() {
        hgr2.add_edge(edge.source(), edge.target());
    }

    // Compute module_weight2
    let mut module_weight2 = vec![0u32; num_modules];
    for (i, &v) in cell_list.iter().enumerate() {
        let idx = v.index();
        module_weight2[i] = if idx < module_weight.len() {
            module_weight[idx]
        } else {
            1
        };
    }
    for (i, &net) in clusters.iter().enumerate() {
        module_weight2[num_cells + i] = *cluster_weight.get(&net).unwrap_or(&0);
    }

    // Set up node_down_list
    let mut node_down_list: Vec<usize> = cell_list.iter().map(|v| v.index()).collect();
    for &net in &clusters {
        if let Some(first_nbr) = hyprgraph.neighbors(net).next() {
            node_down_list.push(first_nbr.index());
        }
    }

    hgr2.clusters = clusters.iter().map(|v| v.index()).collect();
    hgr2.node_down_list = node_down_list;
    hgr2.module_weight = module_weight2.clone();
    // hgr2.parent should be set by caller

    (hgr2, module_weight2)
}

fn construct_graph(
    hyprgraph: &impl Hypergraph<Node = NodeIndex>,
    nets: &[NodeIndex],
    cell_list: &[NodeIndex],
    clusters: &[NodeIndex],
) -> petgraph::graph::Graph<(), (), petgraph::Undirected> {
    let num_cells = cell_list.len();
    let num_clusters = clusters.len();
    let num_modules = num_cells + num_clusters;
    let num_nets = nets.len();
    let total = num_modules + num_nets;

    let mut ugraph = petgraph::graph::Graph::new_undirected();
    for _ in 0..total {
        ugraph.add_node(());
    }

    // Build node_up_map
    let mut node_up_map: HashMap<usize, usize> = HashMap::new();
    for (i_v, &net) in clusters.iter().enumerate() {
        for v in hyprgraph.neighbors(net) {
            node_up_map.insert(v.index(), num_cells + i_v);
        }
    }
    for (i_v, &v) in cell_list.iter().enumerate() {
        node_up_map.insert(v.index(), i_v);
    }

    for (i_net, &net) in nets.iter().enumerate() {
        for v in hyprgraph.neighbors(net) {
            if let Some(&mapped_v) = node_up_map.get(&v.index()) {
                ugraph.add_edge(
                    NodeIndex::new(mapped_v),
                    NodeIndex::new(num_modules + i_net),
                    (),
                );
            }
        }
    }

    ugraph
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;

    #[test]
    fn test_contract_subgraph_basic() {
        let mut netlist = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = netlist.gr.node_indices().collect();
        netlist.add_edge(nodes[0], nodes[4]);
        netlist.add_edge(nodes[1], nodes[4]);
        netlist.add_edge(nodes[2], nodes[5]);
        netlist.add_edge(nodes[3], nodes[5]);
        let weights = vec![1u32, 1, 1, 1];

        let (hgr2, weights2) = contract_subgraph(&netlist, &weights, &HashSet::new());
        assert!(hgr2.number_of_modules() < 4);
        assert_eq!(hgr2.number_of_modules(), weights2.len());
    }

    #[test]
    fn test_contract_subgraph_no_nets() {
        let netlist = SimpleNetlist::new(4, 0);
        let weights = vec![1u32; 4];
        let (hgr2, weights2) = contract_subgraph(&netlist, &weights, &HashSet::new());
        assert_eq!(hgr2.number_of_modules(), 4);
        assert_eq!(weights2.len(), 4);
    }
}
