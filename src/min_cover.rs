use std::collections::{HashMap, HashSet};

use netlistx_rs::min_maximal_matching;
use petgraph::graph::NodeIndex;

use crate::hier_netlist::HierNetlist;
use crate::hypergraph::Hypergraph;

/// Build a `netlistx_rs::Netlist` from a `Hypergraph` so we can use netlistx_rs algorithms.
fn to_netlist(hyprgraph: &impl Hypergraph<Node = NodeIndex>) -> netlistx_rs::Netlist {
    let mut nl = netlistx_rs::Netlist::new();
    let mut mod_map: HashMap<usize, String> = HashMap::new();
    for v in hyprgraph.modules() {
        let name = format!("m{}", v.index());
        mod_map.insert(v.index(), name.clone());
        let _ = nl.add_module(name);
    }
    let mut net_map: HashMap<usize, String> = HashMap::new();
    for net in hyprgraph.nets() {
        let name = format!("n{}", net.index());
        net_map.insert(net.index(), name.clone());
        let _ = nl.add_net(name);
    }
    for net in hyprgraph.nets() {
        let net_name = net_map.get(&net.index()).unwrap();
        for v in hyprgraph.neighbors(net) {
            let mod_name = mod_map.get(&v.index()).unwrap();
            let _ = nl.add_edge(net_name, mod_name);
        }
    }
    nl
}

/// Contract a subgraph by clustering connected modules.
///
/// Each net $n$ becomes a cluster with weight equal to the sum of its module weights:
///
/// $$ w(c_n) = \sum_{v \in N(n)} w(v) $$
///
/// Uses the primal-dual minimum-weight maximal matching algorithm
/// (`min_maximal_matching` from `netlistx_rs`) to select cluster seeds.
/// This is the same algorithm as the C++ version in `netlistx-cpp`.
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

    // Step 2: Find matching via primal-dual minimum-weight maximal matching
    let nl = to_netlist(hyprgraph);
    let mut weight_named = HashMap::new();
    for net in hyprgraph.nets() {
        let name = format!("n{}", net.index());
        let w = cluster_weight.get(&net).copied().unwrap_or(0);
        weight_named.insert(name, w);
    }
    let mut matchset_str: HashSet<String> = HashSet::new();
    let mut dep_str: HashSet<String> = HashSet::new();
    for &f in forbid {
        dep_str.insert(format!("n{}", f.index()));
    }
    let (matched_str, _cost) =
        min_maximal_matching(&nl, &weight_named, &mut matchset_str, &mut dep_str);
    let matched_nets_set: HashSet<NodeIndex> = matched_str
        .iter()
        .filter_map(|name| {
            if name.starts_with('n') {
                name[1..].parse::<usize>().ok().map(NodeIndex::new)
            } else {
                None
            }
        })
        .collect();

    // Step 3: Separate clusters (matched nets) and remaining nets
    let mut clusters: Vec<NodeIndex> = Vec::new();
    let mut remaining_nets: Vec<NodeIndex> = Vec::new();
    let mut covered_modules: HashSet<usize> = HashSet::new();
    for &net in hyprgraph.nets().collect::<Vec<_>>().iter() {
        if matched_nets_set.contains(&net) {
            clusters.push(net);
            for v in hyprgraph.neighbors(net) {
                covered_modules.insert(v.index());
            }
        } else {
            remaining_nets.push(net);
        }
    }

    // Step 4: Build cell list (modules not covered by any cluster)
    let cell_list: Vec<NodeIndex> = hyprgraph
        .modules()
        .filter(|v| !covered_modules.contains(&v.index()))
        .collect();

    let num_cells = cell_list.len();
    let num_clusters = clusters.len();
    let num_modules = num_cells + num_clusters;

    // Step 5: Construct intermediate bipartite graph
    let ugraph = construct_graph(hyprgraph, &remaining_nets, &cell_list, &clusters);
    let _num_remaining_nets = remaining_nets.len();

    // Step 6: Purge duplicate nets (MinHash) and reconstruct graph
    let (gr2, net_weight_map, num_purged_nets) = reconstruct_graph(
        hyprgraph,
        &ugraph,
        &remaining_nets,
        num_clusters,
        num_modules,
    );

    // Step 7: Build HierNetlist from purged graph
    let mut hgr2 = HierNetlist::new(num_modules, num_purged_nets);
    for edge in gr2.raw_edges() {
        hgr2.add_edge(edge.source(), edge.target());
    }

    // Set net weights from duplicate merging
    for (i, w) in &net_weight_map {
        hgr2.net_weight.insert(*i, *w);
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
    // Populate cluster_modules: for each cluster (original net), list all module indices
    let cluster_modules: Vec<Vec<usize>> = clusters
        .iter()
        .map(|&net| hyprgraph.neighbors(net).map(|v| v.index()).collect())
        .collect();
    hgr2.cluster_modules = cluster_modules;

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

// ── MinHash duplicate net pruning ─────────────────────────────────

const LOW_PIN_NET_THRESHOLD: usize = 5;
const MINHASH_SIG_SIZE: usize = 64;
const MINHASH_SIMILARITY: f64 = 0.8;
const MINHASH_MAX_DEGREE: usize = 200;

type MinHashSig = [u64; MINHASH_SIG_SIZE];

fn hash_with_seed(x: u32, seed: u64) -> u64 {
    let mut h = seed;
    h ^= u64::from(x).wrapping_add(0x9e3779b97f4a7c15) + (h << 6) + (h >> 2);
    h
}

fn minhash_signature(
    ugraph: &petgraph::Graph<(), (), petgraph::Undirected>,
    net: NodeIndex,
) -> MinHashSig {
    let mut sig = [u64::MAX; MINHASH_SIG_SIZE];
    for v in ugraph.neighbors(net) {
        let x = v.index() as u32;
        for (i, s) in sig.iter_mut().enumerate() {
            let h = hash_with_seed(x, i as u64);
            if h < *s {
                *s = h;
            }
        }
    }
    sig
}

fn jaccard_similarity(sig1: &MinHashSig, sig2: &MinHashSig) -> f64 {
    let matches = sig1.iter().zip(sig2.iter()).filter(|(a, b)| a == b).count();
    matches as f64 / MINHASH_SIG_SIZE as f64
}

/// Purge duplicate nets (nets connecting the same set of modules).
/// Ported from C++ `purge_duplicate_nets()` in `min_cover.cpp`.
fn purge_duplicate_nets(
    hyprgraph: &impl Hypergraph<Node = NodeIndex>,
    ugraph: &petgraph::Graph<(), (), petgraph::Undirected>,
    nets: &[NodeIndex],
    num_clusters: usize,
    num_modules: usize,
) -> (HashMap<usize, u32>, Vec<usize>) {
    let num_nets = nets.len();
    let mut net_weight: HashMap<usize, u32> = HashMap::new();

    for (i_net, &net) in nets.iter().enumerate() {
        let wt = hyprgraph.get_net_weight(net);
        if wt != 1 {
            net_weight.insert(num_modules + i_net, wt);
        }
    }

    let mut removelist: HashSet<usize> = HashSet::new();
    let mut sig_cache: HashMap<usize, MinHashSig> = HashMap::new();

    for cluster_idx in (num_modules - num_clusters)..num_modules {
        let cluster_node = NodeIndex::new(cluster_idx);
        let net_nodes: Vec<_> = ugraph.neighbors(cluster_node).collect();

        for &net1 in &net_nodes {
            let n1 = net1.index();
            if n1 < num_modules || n1 >= num_modules + num_nets {
                continue;
            }

            if ugraph.neighbors(net1).count() == 1 {
                removelist.insert(n1);
                continue;
            }

            for &net2 in &net_nodes {
                if net2 == net1 {
                    continue;
                }
                let n2 = net2.index();
                let deg1 = ugraph.neighbors(net1).count();
                let deg2 = ugraph.neighbors(net2).count();
                if deg1 != deg2 {
                    continue;
                }

                let mut same = false;
                let deg = deg1;
                if deg <= LOW_PIN_NET_THRESHOLD {
                    let set1: Vec<_> = ugraph.neighbors(net1).collect();
                    let set2: Vec<_> = ugraph.neighbors(net2).collect();
                    if set1.len() == set2.len() {
                        same = set1.iter().all(|v| set2.contains(v));
                    }
                } else if deg <= MINHASH_MAX_DEGREE {
                    // Compute/cache signatures, clone to avoid double-borrow
                    let s1 = *sig_cache
                        .entry(n1)
                        .or_insert_with(|| minhash_signature(ugraph, net1));
                    let s2 = *sig_cache
                        .entry(n2)
                        .or_insert_with(|| minhash_signature(ugraph, net2));
                    let sim = jaccard_similarity(&s1, &s2);
                    if sim >= MINHASH_SIMILARITY {
                        let set1: Vec<_> = ugraph.neighbors(net1).collect();
                        let set2: Vec<_> = ugraph.neighbors(net2).collect();
                        if set1.len() == set2.len() {
                            same = set1.iter().all(|v| set2.contains(v));
                        }
                    }
                }

                if same {
                    removelist.insert(n2);
                    let w1 = net_weight.get(&n1).copied().unwrap_or(1);
                    let w2 = net_weight.get(&n2).copied().unwrap_or(1);
                    net_weight.insert(n1, w1 + w2);
                }
            }
        }
    }

    let updated_nets: Vec<usize> = (num_modules..num_modules + num_nets)
        .filter(|i| !removelist.contains(i))
        .collect();

    (net_weight, updated_nets)
}

/// Reconstruct graph after purging duplicate nets.
/// Ported from C++ `reconstruct_graph()` in `min_cover.cpp`.
fn reconstruct_graph(
    hyprgraph: &impl Hypergraph<Node = NodeIndex>,
    ugraph: &petgraph::Graph<(), (), petgraph::Undirected>,
    nets: &[NodeIndex],
    num_clusters: usize,
    num_modules: usize,
) -> (
    petgraph::Graph<(), (), petgraph::Undirected>,
    HashMap<usize, u32>,
    usize,
) {
    let (net_weight, updated_nets) =
        purge_duplicate_nets(hyprgraph, ugraph, nets, num_clusters, num_modules);

    let num_nets = updated_nets.len();
    let mut gr2: petgraph::Graph<(), (), petgraph::Undirected> = petgraph::Graph::new_undirected();
    for _ in 0..num_modules + num_nets {
        gr2.add_node(());
    }

    for (i_net, &net) in updated_nets.iter().enumerate() {
        for v in ugraph.neighbors(NodeIndex::new(net)) {
            gr2.add_edge(v, NodeIndex::new(num_modules + i_net), ());
        }
    }

    let net_weight2: HashMap<usize, u32> = updated_nets
        .iter()
        .enumerate()
        .filter_map(|(i, &net)| net_weight.get(&net).map(|&w| (i, w)))
        .collect();

    (gr2, net_weight2, num_nets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;

    /// Greedy matching (old): sort nets by descending weight, pick non-overlapping.
    fn greedy_matching(
        hyprgraph: &impl Hypergraph<Node = NodeIndex>,
        cluster_weight: &HashMap<NodeIndex, u32>,
        forbid: &HashSet<NodeIndex>,
    ) -> HashSet<NodeIndex> {
        let mut sorted: Vec<_> = cluster_weight.iter().collect();
        sorted.sort_by_key(|(_, &w)| std::cmp::Reverse(w));
        let mut matched = HashSet::new();
        let mut covered: HashSet<usize> = HashSet::new();
        for (&net, _) in &sorted {
            if forbid.contains(&net) {
                continue;
            }
            let idxs: Vec<usize> = hyprgraph.neighbors(net).map(|v| v.index()).collect();
            if idxs.len() < 2 {
                continue;
            }
            if idxs.iter().any(|i| covered.contains(i)) {
                continue;
            }
            matched.insert(net);
            for i in idxs {
                covered.insert(i);
            }
        }
        matched
    }

    #[test]
    fn test_matching_comparison_p1() {
        // Read p1.json and compare greedy vs primal-dual matching.
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("testcases/p1.json");
        let nl = netlistx_rs::read_node_link_json(&path).expect("read p1.json");
        let hg = crate::netlist_adapter::NetlistHypergraph::from_netlist(&nl);
        let module_weight: Vec<u32> = hg.modules().map(|v| hg.get_module_weight(v)).collect();

        let cluster_weight: HashMap<_, u32> = hg
            .nets()
            .map(|net| {
                let w: u32 = hg
                    .neighbors(net)
                    .map(|v| {
                        let i = v.index();
                        if i < module_weight.len() {
                            module_weight[i]
                        } else {
                            1
                        }
                    })
                    .sum();
                (net, w)
            })
            .collect();

        let forbid = HashSet::new();
        let greedy_set = greedy_matching(&hg, &cluster_weight, &forbid);
        let netlist_nl = super::to_netlist(&hg);
        let mut wgt = HashMap::new();
        for net in hg.nets() {
            wgt.insert(
                format!("n{}", net.index()),
                *cluster_weight.get(&net).unwrap_or(&0),
            );
        }
        let mut ms = std::collections::HashSet::new();
        let mut dp = std::collections::HashSet::new();
        let (pd_set, _) = netlistx_rs::min_maximal_matching(&netlist_nl, &wgt, &mut ms, &mut dp);

        let n_mod = hg.number_of_modules();
        let greedy_clusters = greedy_set.len();
        let pd_clusters = pd_set.len();
        let greedy_covered: HashSet<usize> = greedy_set
            .iter()
            .flat_map(|&n| hg.neighbors(n).map(|v| v.index()).collect::<Vec<_>>())
            .collect();
        let pd_covered: HashSet<usize> = pd_set
            .iter()
            .filter_map(|name| {
                if name.starts_with('n') {
                    name[1..].parse::<usize>().ok()
                } else {
                    None
                }
            })
            .flat_map(|idx| {
                hg.neighbors(NodeIndex::new(idx))
                    .map(|v| v.index())
                    .collect::<Vec<_>>()
            })
            .collect();

        eprintln!("\n=== p1 clustering comparison ===");
        eprintln!("  Modules: {}", n_mod);
        eprintln!("  Nets: {}", hg.nets().count());
        eprintln!(
            "  Greedy:  {} clusters, {} modules covered ({} uncovered)",
            greedy_clusters,
            greedy_covered.len(),
            n_mod - greedy_covered.len()
        );
        eprintln!(
            "  PrimalDual: {} clusters, {} modules covered ({} uncovered)",
            pd_clusters,
            pd_covered.len(),
            n_mod - pd_covered.len()
        );
    }

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
