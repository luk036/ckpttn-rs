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
