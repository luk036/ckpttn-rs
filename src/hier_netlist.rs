use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::hypergraph::Hypergraph;

/// Hierarchical netlist for multi-level partitioning.
///
/// Extends a base netlist with cluster tracking, net weights,
/// and projection methods (up/down) for multi-level graph coarsening
/// and uncoarsening.
/// Ported from Python `HierNetlist` in `HierNetlist.py`.
pub struct HierNetlist {
    /// Parent netlist (the level above)
    pub parent: Option<Box<HierNetlist>>,
    /// Number of modules
    pub num_modules: usize,
    /// Graph representation (bipartite: modules + nets)
    pub gr: petgraph::graph::Graph<(), (), petgraph::Undirected>,
    /// Module weights
    pub module_weight: Vec<u32>,
    /// Net weights
    pub net_weight: HashMap<usize, u32>,
    /// Cluster tracking for projection down
    pub clusters: Vec<usize>,
    /// Node list for projection down
    pub node_down_list: Vec<usize>,
}

impl HierNetlist {
    pub fn new(num_modules: usize, num_nets: usize) -> Self {
        let total = num_modules + num_nets;
        let mut gr = petgraph::graph::Graph::new_undirected();
        for _ in 0..total {
            gr.add_node(());
        }
        HierNetlist {
            parent: None,
            num_modules,
            gr,
            module_weight: vec![1; num_modules],
            net_weight: HashMap::new(),
            clusters: Vec::new(),
            node_down_list: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, module: NodeIndex, net: NodeIndex) {
        self.gr.add_edge(module, net, ());
    }

    pub fn number_of_modules(&self) -> usize {
        self.num_modules
    }

    pub fn number_of_nets(&self) -> usize {
        self.gr.node_count() - self.num_modules
    }

    pub fn number_of_pins(&self) -> usize {
        let mut count = 0;
        for net_idx in self.num_modules..self.gr.node_count() {
            count += self.gr.neighbors(NodeIndex::new(net_idx)).count();
        }
        count
    }

    pub fn get_max_degree(&self) -> usize {
        (0..self.num_modules)
            .map(|i| self.gr.neighbors(NodeIndex::new(i)).count())
            .max()
            .unwrap_or(0)
    }

    pub fn projection_down(&self, part: &[u8], part_down: &mut [u8]) {
        let num_cells = self.node_down_list.len() - self.clusters.len();
        for (v1, &v2) in self.node_down_list[..num_cells].iter().enumerate() {
            if v1 < part.len() && v2 < part_down.len() {
                part_down[v2] = part[v1];
            }
        }
        for (i_v, &net) in self.clusters.iter().enumerate() {
            let p = part[num_cells + i_v];
            if let Some(ref parent) = self.parent {
                for v2 in parent.gr.neighbors(NodeIndex::new(net)) {
                    let idx = v2.index();
                    if idx < part_down.len() {
                        part_down[idx] = p;
                    }
                }
            }
        }
    }

    pub fn projection_up(&self, part: &[u8], part_up: &mut [u8]) {
        for (v1, &v2) in self.node_down_list.iter().enumerate() {
            if v2 < part.len() && v1 < part_up.len() {
                part_up[v1] = part[v2];
            }
        }
    }
}

impl Hypergraph for HierNetlist {
    type Node = NodeIndex;

    fn modules(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        Box::new((0..self.num_modules).map(NodeIndex::new))
    }

    fn nets(&self) -> Box<dyn Iterator<Item = Self::Node> + '_> {
        let total = self.gr.node_count();
        Box::new((self.num_modules..total).map(NodeIndex::new))
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

    fn get_net_weight(&self, net: Self::Node) -> u32 {
        let idx = net.index();
        *self.net_weight.get(&idx).unwrap_or(&1)
    }

    fn number_of_modules(&self) -> usize {
        self.num_modules
    }

    fn get_max_degree(&self) -> usize {
        self.get_max_degree()
    }

    fn module_index(&self, v: Self::Node) -> usize {
        v.index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hier_netlist_new() {
        let hn = HierNetlist::new(4, 2);
        assert_eq!(hn.num_modules, 4);
        assert_eq!(hn.gr.node_count(), 6);
        assert_eq!(hn.module_weight.len(), 4);
        assert!(hn.parent.is_none());
    }

    #[test]
    fn test_hier_netlist_counts() {
        let hn = HierNetlist::new(4, 2);
        assert_eq!(hn.number_of_modules(), 4);
        assert_eq!(hn.number_of_nets(), 2);
        assert_eq!(hn.number_of_pins(), 0);
        assert_eq!(hn.get_max_degree(), 0);
    }

    #[test]
    fn test_hier_netlist_add_edge() {
        let mut hn = HierNetlist::new(4, 2);
        let module = NodeIndex::new(0);
        let net = NodeIndex::new(4);
        hn.add_edge(module, net);
        assert_eq!(hn.number_of_pins(), 1);
        assert_eq!(hn.get_max_degree(), 1);
    }

    #[test]
    fn test_hier_netlist_hypergraph_trait() {
        let mut hn = HierNetlist::new(4, 2);
        let module = NodeIndex::new(0);
        let net = NodeIndex::new(4);
        hn.add_edge(module, net);

        let modules: Vec<_> = hn.modules().collect();
        assert_eq!(modules.len(), 4);

        let nets: Vec<_> = hn.nets().collect();
        assert_eq!(nets.len(), 2);

        let weight = hn.get_module_weight(NodeIndex::new(0));
        assert_eq!(weight, 1);

        let net_weight = hn.get_net_weight(NodeIndex::new(4));
        assert_eq!(net_weight, 1);

        let deg = hn.degree(net);
        assert_eq!(deg, 1);

        let idx = hn.module_index(module);
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_hier_netlist_projection_down() {
        let mut hn = HierNetlist::new(4, 2);
        hn.node_down_list = vec![0, 1, 100, 101];
        hn.clusters = vec![101];

        let part = vec![0u8, 1, 2, 3];
        let mut part_down = vec![0u8; 4];
        hn.projection_down(&part, &mut part_down);
    }

    #[test]
    fn test_hier_netlist_projection_up() {
        let hn = HierNetlist::new(4, 2);
        let part = vec![0u8, 1, 2, 3];
        let mut part_up = vec![0u8; 4];
        hn.projection_up(&part, &mut part_up);
    }

    #[test]
    fn test_hier_netlist_weight_out_of_range() {
        let hn = HierNetlist::new(2, 1);
        // module_index beyond module_weight.len() returns default 1
        let w = hn.get_module_weight(NodeIndex::new(5));
        assert_eq!(w, 1);
    }
}
