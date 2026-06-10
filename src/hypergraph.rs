use petgraph::graph::NodeIndex;

/// Trait abstracting a hypergraph (netlist) for partitioning algorithms.
///
/// Equivalent to the `Gnl` concept in the C++ codebase.
pub trait Hypergraph {
    /// Type used to identify nodes (modules and nets)
    type Node: Copy + Eq + std::hash::Hash;

    /// Iterate over all modules in the hypergraph
    fn modules(&self) -> Box<dyn Iterator<Item = Self::Node> + '_>;

    /// Iterate over all nets in the hypergraph
    fn nets(&self) -> Box<dyn Iterator<Item = Self::Node> + '_>;

    /// Get the neighbors (vertices connected by a net)
    fn neighbors(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_>;

    /// Get the degree of a node (number of incident nets for a module, or number of
    /// modules for a net)
    fn degree(&self, node: Self::Node) -> usize;

    /// Get the weight of a module
    fn get_module_weight(&self, v: Self::Node) -> u32;

    /// Get the weight of a net (default 1)
    fn get_net_weight(&self, _net: Self::Node) -> u32 {
        1
    }

    /// Number of modules
    fn number_of_modules(&self) -> usize;

    /// Maximum degree among all modules
    fn get_max_degree(&self) -> usize;

    /// Get the index of a module for use in partition arrays.
    /// In the C++ code, node_t IS the module index (integer type),
    /// so this maps node → usize for array indexing.
    fn module_index(&self, v: Self::Node) -> usize;
}

/// A simple netlist backed by petgraph.
///
/// Usable as a test/example hypergraph for partitioning.
pub struct SimpleNetlist {
    /// Number of modules (first `num_modules` nodes in the graph)
    pub num_modules: usize,
    /// Graph representation
    pub gr: petgraph::Graph<(), (), petgraph::Undirected>,
    /// Module weights
    pub module_weight: Vec<u32>,
}

impl SimpleNetlist {
    pub fn new(num_modules: usize, num_nets: usize) -> Self {
        let total = num_modules + num_nets;
        let mut gr = petgraph::Graph::new_undirected();
        for _ in 0..total {
            gr.add_node(());
        }
        SimpleNetlist {
            num_modules,
            gr,
            module_weight: vec![1; num_modules],
        }
    }

    pub fn add_edge(&mut self, module: NodeIndex, net: NodeIndex) {
        self.gr.add_edge(module, net, ());
    }
}

impl Hypergraph for SimpleNetlist {
    type Node = NodeIndex;

    fn modules(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.gr.node_indices().take(self.num_modules))
    }

    fn nets(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.gr.node_indices().skip(self.num_modules))
    }

    fn neighbors(&self, node: Self::Node) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new(self.gr.neighbors(node))
    }

    fn degree(&self, node: Self::Node) -> usize {
        self.gr.neighbors(node).count()
    }

    fn get_module_weight(&self, v: Self::Node) -> u32 {
        let idx = v.index();
        if idx < self.module_weight.len() {
            self.module_weight[idx]
        } else {
            1
        }
    }

    fn number_of_modules(&self) -> usize {
        self.num_modules
    }

    fn get_max_degree(&self) -> usize {
        self.modules().map(|v| self.degree(v)).max().unwrap_or(0)
    }

    fn module_index(&self, v: Self::Node) -> usize {
        v.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::NodeIndex;

    #[test]
    fn test_simple_netlist_new_sizes() {
        let nl = SimpleNetlist::new(4, 2);
        assert_eq!(nl.num_modules, 4);
        assert_eq!(nl.gr.node_count(), 6);
        assert_eq!(nl.module_weight.len(), 4);
        assert_eq!(nl.module_weight, vec![1; 4]);
    }

    #[test]
    fn test_add_edge_and_degree() {
        let mut nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        nl.add_edge(nodes[0], nodes[4]);
        nl.add_edge(nodes[1], nodes[4]);
        assert_eq!(nl.degree(nodes[4]), 2);
        assert_eq!(nl.degree(nodes[0]), 1);
        assert_eq!(nl.degree(nodes[1]), 1);
        assert_eq!(nl.degree(nodes[2]), 0);
    }

    #[test]
    fn test_modules_iter() {
        let nl = SimpleNetlist::new(3, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        let modules: Vec<_> = nl.modules().collect();
        assert_eq!(modules.len(), 3);
        assert_eq!(modules, nodes[..3]);
    }

    #[test]
    fn test_nets_iter() {
        let nl = SimpleNetlist::new(3, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        let nets: Vec<_> = nl.nets().collect();
        assert_eq!(nets.len(), 2);
        assert_eq!(nets, nodes[3..]);
    }

    #[test]
    fn test_neighbors_basic() {
        let mut nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        nl.add_edge(nodes[0], nodes[4]);
        let nbrs: Vec<_> = nl.neighbors(nodes[0]).collect();
        assert_eq!(nbrs, vec![nodes[4]]);
    }

    #[test]
    fn test_get_module_weight_in_bounds() {
        let nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        assert_eq!(nl.get_module_weight(nodes[0]), 1);
        assert_eq!(nl.get_module_weight(nodes[3]), 1);
    }

    #[test]
    fn test_get_module_weight_out_of_bounds() {
        let nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        assert_eq!(nl.get_module_weight(nodes[5]), 1);
    }

    #[test]
    fn test_get_net_weight_default() {
        let nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        assert_eq!(nl.get_net_weight(nodes[4]), 1);
    }

    #[test]
    fn test_number_of_modules() {
        let nl = SimpleNetlist::new(4, 2);
        assert_eq!(nl.number_of_modules(), 4);
        let nl0 = SimpleNetlist::new(0, 0);
        assert_eq!(nl0.number_of_modules(), 0);
    }

    #[test]
    fn test_get_max_degree_normal() {
        let mut nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        nl.add_edge(nodes[0], nodes[4]);
        nl.add_edge(nodes[0], nodes[5]);
        nl.add_edge(nodes[1], nodes[4]);
        assert_eq!(nl.get_max_degree(), 2);
    }

    #[test]
    fn test_get_max_degree_empty() {
        let nl = SimpleNetlist::new(0, 0);
        assert_eq!(nl.get_max_degree(), 0);
    }

    #[test]
    fn test_module_index() {
        let nl = SimpleNetlist::new(4, 2);
        let nodes: Vec<NodeIndex> = nl.gr.node_indices().collect();
        assert_eq!(nl.module_index(nodes[0]), 0);
        assert_eq!(nl.module_index(nodes[2]), 2);
    }
}
