use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::hypergraph::Hypergraph;

/// Adapter that wraps `netlistx_rs::Netlist` and implements the `Hypergraph` trait.
///
/// Maps named modules to indices 0..num_modules and named nets to indices
/// num_modules..num_modules+num_nets, enabling netlistx-rs netlists to be
/// used with ckpttn-rs partitioning algorithms.
pub struct NetlistHypergraph {
    num_modules: usize,
    num_nets: usize,
    module_weights: Vec<u32>,
    /// Adjacency: for each module index, list of (net_index, net_name)
    module_to_nets: Vec<Vec<(usize, String)>>,
    /// Adjacency: for each net index, list of (module_index, module_name)
    net_to_modules: Vec<Vec<(usize, String)>>,
    /// Maximum module degree
    max_degree: usize,
}

impl NetlistHypergraph {
    pub fn from_netlist(nl: &netlistx_rs::Netlist) -> Self {
        let num_modules = nl.num_modules();
        let num_nets = nl.num_nets();

        let module_to_idx: HashMap<&str, usize> = nl
            .modules
            .iter()
            .enumerate()
            .map(|(i, name)| (name.as_str(), i))
            .collect();

        let mut module_weights = vec![1u32; num_modules];
        for (name, idx) in &module_to_idx {
            let w = nl.get_module_weight(name);
            if w > 0 {
                module_weights[*idx] = w as u32;
            }
        }

        let mut module_to_nets: Vec<Vec<(usize, String)>> = vec![Vec::new(); num_modules];
        let mut net_to_modules: Vec<Vec<(usize, String)>> = vec![Vec::new(); num_nets];

        let mut max_deg = 0usize;

        for edge in nl.grph.raw_edges() {
            let s = edge.source().index();
            let t = edge.target().index();
            let (mod_idx, net_global_idx) = if s < num_modules {
                (s, t)
            } else {
                (t, s)
            };
            if net_global_idx < num_modules {
                continue;
            }
            let local_net_idx = net_global_idx - num_modules;
            if local_net_idx >= num_nets {
                continue;
            }
            let net_name = nl.grph[NodeIndex::new(net_global_idx)].clone();
            let mod_name = nl.grph[NodeIndex::new(mod_idx)].clone();
            module_to_nets[mod_idx].push((net_global_idx, net_name));
            net_to_modules[local_net_idx].push((mod_idx, mod_name));
        }

        for m in &module_to_nets {
            if m.len() > max_deg {
                max_deg = m.len();
            }
        }

        NetlistHypergraph {
            num_modules,
            num_nets,
            module_weights,
            module_to_nets,
            net_to_modules,
            max_degree: max_deg,
        }
    }
}

impl Hypergraph for NetlistHypergraph {
    type Node = NodeIndex;

    fn modules(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new((0..self.num_modules).map(NodeIndex::new))
    }

    fn nets(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(
            (self.num_modules..self.num_modules + self.num_nets).map(NodeIndex::new),
        )
    }

    fn neighbors(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        let idx = node.index();
        if idx < self.num_modules {
            // Module → return connected nets
            let nets: Vec<_> = self.module_to_nets[idx]
                .iter()
                .map(|(net_idx, _)| NodeIndex::new(*net_idx))
                .collect();
            Box::new(nets.into_iter())
        } else {
            // Net → return connected modules
            let local_net_idx = idx - self.num_modules;
            let mods: Vec<_> = self.net_to_modules[local_net_idx]
                .iter()
                .map(|(mod_idx, _)| NodeIndex::new(*mod_idx))
                .collect();
            Box::new(mods.into_iter())
        }
    }

    fn degree(&self, node: Self::Node) -> usize {
        let idx = node.index();
        if idx < self.num_modules {
            self.module_to_nets[idx].len()
        } else {
            let local_net_idx = idx - self.num_modules;
            self.net_to_modules[local_net_idx].len()
        }
    }

    fn get_module_weight(&self, v: Self::Node) -> u32 {
        let idx = v.index();
        if idx < self.module_weights.len() {
            self.module_weights[idx]
        } else {
            1
        }
    }

    fn number_of_modules(&self) -> usize {
        self.num_modules
    }

    fn get_max_degree(&self) -> usize {
        self.max_degree
    }

    fn module_index(&self, v: Self::Node) -> usize {
        v.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use netlistx_rs::Netlist;

    fn make_simple_netlist() -> Netlist {
        let mut nl = Netlist::new();
        nl.add_module("m0".to_string()).unwrap();
        nl.add_module("m1".to_string()).unwrap();
        nl.add_module("m2".to_string()).unwrap();
        nl.add_module("m3".to_string()).unwrap();
        nl.add_net("n0".to_string()).unwrap();
        nl.add_net("n1".to_string()).unwrap();
        nl.add_edge("n0", "m0").unwrap();
        nl.add_edge("n0", "m1").unwrap();
        nl.add_edge("n1", "m2").unwrap();
        nl.add_edge("n1", "m3").unwrap();
        nl
    }

    #[test]
    fn test_netlist_hypergraph_from_netlist() {
        let nl = make_simple_netlist();
        let hg = NetlistHypergraph::from_netlist(&nl);
        assert_eq!(hg.number_of_modules(), 4);
        assert_eq!(hg.nets().count(), 2);
    }

    #[test]
    fn test_netlist_hypergraph_module_weights() {
        let mut nl = make_simple_netlist();
        nl.set_module_weight("m0", 5);
        let hg = NetlistHypergraph::from_netlist(&nl);
        assert_eq!(hg.get_module_weight(NodeIndex::new(0)), 5);
        assert_eq!(hg.get_module_weight(NodeIndex::new(1)), 1);
    }

    #[test]
    fn test_netlist_hypergraph_degree() {
        let nl = make_simple_netlist();
        let hg = NetlistHypergraph::from_netlist(&nl);
        // m0 connected to n0 only
        assert_eq!(hg.degree(NodeIndex::new(0)), 1);
        // n0 connected to m0, m1
        assert_eq!(hg.degree(NodeIndex::new(4)), 2);
    }

    #[test]
    fn test_netlist_hypergraph_neighbors() {
        let nl = make_simple_netlist();
        let hg = NetlistHypergraph::from_netlist(&nl);
        let nbrs: Vec<_> = hg.neighbors(NodeIndex::new(0)).collect();
        // m0 connected to n0 (index 4)
        assert!(nbrs.contains(&NodeIndex::new(4)));
        assert_eq!(nbrs.len(), 1);
    }

    #[test]
    fn test_netlist_hypergraph_max_degree() {
        let mut nl = Netlist::new();
        nl.add_module("m0".to_string()).unwrap();
        nl.add_module("m1".to_string()).unwrap();
        nl.add_module("m2".to_string()).unwrap();
        nl.add_net("n0".to_string()).unwrap();
        nl.add_edge("n0", "m0").unwrap();
        nl.add_edge("n0", "m1").unwrap();
        nl.add_edge("n0", "m2").unwrap();
        let hg = NetlistHypergraph::from_netlist(&nl);
        assert_eq!(hg.get_max_degree(), 1); // each module connected to 1 net
    }
}
