use petgraph::Graph;
use std::collections::{HashMap, HashSet};

struct Netlist<'a> {
    num_pads: i32,
    cost_model: i32,
    gra: Graph<&'a str, ()>,
    modules: Vec<&'a str>,
    nets: Vec<&'a str>,
    num_modules: usize,
    num_nets: usize,
    net_weight: Option<HashMap<&'a str, i32>>,
    module_weight: Option<HashMap<&'a str, i32>>,
    module_fixed: HashSet<&'a str>,
    max_degree: u32,
    max_net_degree: u32,
}

fn create_test_netlist<'a>() -> Netlist<'a> {
    let mut gra = Graph::new();
    let a0 = gra.add_node("a0");
    let a1 = gra.add_node("a1");
    let a2 = gra.add_node("a2");
    let a3 = gra.add_node("a3");
    let a4 = gra.add_node("a4");
    let a5 = gra.add_node("a5");
    let module_weight: HashMap<&str, i32> = [("a0", 533), ("a1", 543), ("a2", 532)]
        .iter()
        .cloned()
        .collect();
    gra.extend_with_edges(&[(a3, a0), (a3, a1), (a5, a0)]);
    gra.graph_mut().set_node_count(6);
    gra.graph_mut().set_edge_count(3);
    let modules = vec!["a0", "a1", "a2"];
    let nets = vec!["a3", "a4", "a5"];
    let mut hgr = Netlist {
        num_pads: 0,
        cost_model: 0,
        gra,
        modules,
        nets,
        num_modules: modules.len(),
        num_nets: nets.len(),
        net_weight: None,
        module_weight: None,
        module_fixed: HashSet::new(),
        max_degree: 0,
        max_net_degree: 0,
    };
    hgr.module_weight = Some(module_weight);
    hgr
}
