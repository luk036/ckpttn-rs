use petgraph::Graph;
use std::collections::{HashMap, HashSet};

struct Netlist<'a> {
    num_pads: i32,
    cost_model: i32,
    grpy: Graph<&'a str, ()>,
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
    let mut grpy = Graph::new();
    let a0 = grpy.add_node("a0");
    let a1 = grpy.add_node("a1");
    let a2 = grpy.add_node("a2");
    let a3 = grpy.add_node("a3");
    let a4 = grpy.add_node("a4");
    let a5 = grpy.add_node("a5");
    let module_weight: HashMap<&str, i32> = [("a0", 533), ("a1", 543), ("a2", 532)]
        .iter()
        .cloned()
        .collect();
    grpy.extend_with_edges(&[(a3, a0), (a3, a1), (a5, a0)]);
    grpy.graph_mut().set_node_count(6);
    grpy.graph_mut().set_edge_count(3);
    let modules = vec!["a0", "a1", "a2"];
    let nets = vec!["a3", "a4", "a5"];
    let mut hyprgraph = Netlist {
        num_pads: 0,
        cost_model: 0,
        grpy,
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
    hyprgraph.module_weight = Some(module_weight);
    hyprgraph
}
