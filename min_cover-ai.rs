use std::collections::HashMap;
use std::collections::HashSet;

use petgraph::graph::Graph;
use petgraph::prelude::*;
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use serde_json::json;

use crate::HierNetlist;

#[derive(Clone)]
struct TinyGraph {
    num_nodes: usize,
}

impl TinyGraph {
    fn cheat_node_dict(&self) -> HashMap<usize, HashMap<String, usize>> {
        let mut node_dict = HashMap::new();
        for i in 0..self.num_nodes {
            node_dict.insert(i, HashMap::new());
        }
        node_dict
    }

    fn cheat_adjlist_outer_dict(&self) -> HashMap<usize, HashMap<usize, HashMap<String, usize>>> {
        let mut adjlist_outer_dict = HashMap::new();
        for i in 0..self.num_nodes {
            adjlist_outer_dict.insert(i, HashMap::new());
        }
        adjlist_outer_dict
    }

    fn init_nodes(&mut self, n: usize) {
        self.num_nodes = n;
    }
}

#[derive(Clone)]
struct Netlist {
    gra: Graph<usize, usize>,
    modules: Vec<usize>,
    nets: Vec<usize>,
    num_modules: usize,
    num_nets: usize,
    module_weight: Option<Vec<usize>>,
    module_fixed: HashSet<usize>,
    max_degree: usize,
}

impl Netlist {
    fn new(
        gra: Graph<usize, usize>,
        modules: Vec<usize>,
        nets: Vec<usize>,
    ) -> Self {
        let num_modules = modules.len();
        let num_nets = nets.len();
        let max_degree = modules.iter().map(|&cell| gra.neighbors(cell).count()).max().unwrap_or(0);
        Self {
            gra,
            modules,
            nets,
            num_modules,
            num_nets,
            module_weight: None,
            module_fixed: HashSet::new(),
            max_degree,
        }
    }

    fn number_of_modules(&self) -> usize {
        self.num_modules
    }

    fn number_of_nets(&self) -> usize {
        self.num_nets
    }

    fn number_of_nodes(&self) -> usize {
        self.gra.node_count()
    }

    fn number_of_pins(&self) -> usize {
        self.gra.edge_count()
    }

    fn get_max_degree(&self) -> usize {
        self.max_degree
    }

    fn get_module_weight(&self, v: usize) -> usize {
        self.module_weight.as_ref().unwrap()[v]
    }

    fn get_net_weight(&self, _: usize) -> usize {
        1
    }
}

fn min_maximal_matching(
    hgr: &Netlist,
    weight: &mut HashMap<usize, usize>,
    matchset: Option<&mut HashSet<usize>>,
    dep: Option<&mut HashSet<usize>>,
) -> (HashSet<usize>, usize) {
    let mut matchset = matchset.unwrap_or(&mut HashSet::new());
    let mut dep = dep.unwrap_or(&mut HashSet::new());

    let mut total_primal_cost = 0;
    let mut total_dual_cost = 0;
    let mut gap = weight.clone();

    for &net in &hgr.nets {
        if dep.contains(&net) {
            continue;
        }
        if matchset.contains(&net) {
            continue;
        }
        let mut min_val = gap[&net];
        let mut min_net = net;
        for &vtx in hgr.gra.neighbors(net) {
            for &net2 in hgr.gra.neighbors(vtx) {
                if dep.contains(&net2) {
                    continue;
                }
                if min_val > gap[&net2] {
                    min_val = gap[&net2];
                    min_net = net2;
                }
            }
        }
        dep.extend(hgr.gra.neighbors(net));
        matchset.insert(min_net);
        total_primal_cost += weight[&min_net];
        total_dual_cost += min_val;
        if min_net == net {
            continue;
        }
        gap.entry(net).and_modify(|e| *e -= min_val);
        for &vtx in hgr.gra.neighbors(net) {
            for &net2 in hgr.gra.neighbors(vtx) {
                gap.entry(net2).and_modify(|e| *e -= min_val);
            }
        }
    }
    assert!(total_dual_cost <= total_primal_cost);
    (matchset, total_primal_cost)
}

fn contract_subgraph(
    hgr: &Netlist,
    module_weight: &mut Vec<usize>,
    forbid: &HashSet<usize>,
) -> (HierNetlist, Vec<usize>) {
    let cluster_weight: Vec<usize> = hgr.nets.iter().map(|&net| hgr.gra.neighbors(net).map(|cell| module_weight[cell]).sum()).collect();
    let (clusters, nets, cell_list) = setup(hgr, &cluster_weight, forbid);
    let gra = construct_graph(hgr, &nets, &cell_list, &clusters);
    let num_modules = cell_list.len() + clusters.len();
    let num_clusters = clusters.len();
    let (gr2, net_weight2, num_nets) = reconstruct_graph(hgr, gra, &nets, num_clusters, num_modules);
    let mut nets = HashSet::new();
    let mut hgr2 = HierNetlist::new(gr2, (0..num_modules).collect(), (num_modules..num_modules+num_nets).collect());
    let mut module_weight2 = vec![0; num_modules];
    let num_cells = num_modules - num_clusters;
    for (v, &v2) in cell_list.iter().enumerate() {
        module_weight2[v] = module_weight[v2];
    }
    for (i_v, &net) in clusters.iter().enumerate() {
        module_weight2[num_cells + i_v] = cluster_weight[net];
    }
    let mut node_down_list = cell_list.clone();
    node_down_list.extend(clusters.iter().map(|&net| hgr.gra.neighbors(net).next().unwrap()));
    hgr2.clusters = clusters;
    hgr2.node_down_list = node_down_list;
    hgr2.module_weight = Some(module_weight2.clone());
    hgr2.net_weight = net_weight2;
    hgr2.parent = Some(Box::new(hgr.clone()));
    (hgr2, module_weight2)
}

fn setup(
    hgr: &Netlist,
    cluster_weight: &[usize],
    forbid: &HashSet<usize>,
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let mut clusters = Vec::new();
    let mut nets = Vec::new();
    let mut cell_list = Vec::new();
    for &cell in &hgr.modules {
        if forbid.contains(&cell) {
            cell_list.push(cell);
        } else {
            clusters.push(cell);
        }
    }
    for (i, &net) in hgr.nets.iter().enumerate() {
        if hgr.gra.neighbors(net).all(|cell| forbid.contains(&cell)) {
            continue;
        }
        if cluster_weight[i] == 0 {
            continue;
        }
        nets.push(net);
    }
    (clusters, nets, cell_list)
}

fn construct_graph(
    hgr: &Netlist,
    nets: &[usize],
    cell_list: &[usize],
    clusters: &[usize],
) -> Graph<usize, usize> {
    let mut gra = Graph::new();
    let mut node_map = HashMap::new();
    for &cell in cell_list {
        node_map.insert(cell, gra.add_node(cell));
    }
    for (i, &cluster) in clusters.iter().enumerate() {
        let node = gra.add_node(hgr.num_modules + i);
        for &cell in hgr.gra.neighbors(cluster) {
            if let Some(&node2) = node_map.get(&cell) {
                gra.add_edge(node, node2, i);
            }
        }
    }
    for &net in nets {
        let mut nodes = Vec::new();
        for &cell in hgr.gra.neighbors(net) {
            if let Some(&node) = node_map.get(&cell) {
                nodes.push(node);
            }
        }
        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                gra.add_edge(nodes[i], nodes[j], net);
            }
        }
    }
    gra
}

fn reconstruct_graph(
    hgr: &Netlist,
    gra: Graph<usize, usize>,
    nets: &[usize],
    num_clusters: usize,
    num_modules: usize,
) -> (Graph<usize, usize>, Vec<usize>, usize) {
    let mut net_weight = vec![0; hgr.num_nets];
    for &net in nets {
        let mut weight = 0;
        for &cell in hgr.gra.neighbors(net) {
            if cell < hgr.num_modules {
                weight += 1;
            }
        }
        net_weight[net] = weight;
    }
    let mut gr2 = Graph::new();
    let mut node_map = HashMap::new();
    for i in 0..num_modules {
        let node = gr2.add_node(i);
        node_map.insert(i, node);
    }
    for (i, &net) in nets.iter().enumerate() {
        let mut nodes = Vec::new();
        for &cell in hgr.gra.neighbors(net) {
            if cell < hgr.num_modules {
                if let Some(&node) = node_map.get(&cell) {
                    nodes.push(node);
                }
            } else {
                nodes.push(gr2.add_node(cell - hgr.num_modules + num_modules));
            }
        }
        for i in 0..nodes.len() {
            for j in i+1..nodes.len() {
                gr2.add_edge(nodes[i], nodes[j], net);
            }
        }
    }
    for edge in gra.edge_references() {
        let (u, v) = (edge.source(), edge.target());
        let net = *edge.weight();
        if let Some(&u2) = node_map.get(&u) {
            if let Some(&v2) = node_map.get(&v) {
                gr2.add_edge(u2, v2, net);
            }
        }
    }
    (gr2, net_weight, nets.len())
