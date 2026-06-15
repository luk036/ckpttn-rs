use std::collections::HashMap;

use petgraph::graph::NodeIndex;

use crate::Hypergraph;

/// Multi-FPGA Partitioning Manager.
///
/// Manages partitioning of a design across multiple FPGAs,
/// optimizing for resource utilization and inter-FPGA communication.
/// Ported from Python `MultiFPGAPartMgr` in `MultiFPGAPartMgr.py`.
pub struct MultiFPGAPartMgr {
    pub num_fpgas: u8,
    pub fpga_resources: Vec<HashMap<String, f64>>,
    pub bal_tol: f64,
    pub total_cost: i32,
}

impl MultiFPGAPartMgr {
    pub fn new(num_fpgas: u8, fpga_resources: Vec<HashMap<String, f64>>, bal_tol: f64) -> Self {
        MultiFPGAPartMgr {
            num_fpgas,
            fpga_resources,
            bal_tol,
            total_cost: 0,
        }
    }

    pub fn partition_design(
        &mut self,
        _hyprgraph: &impl Hypergraph<Node = NodeIndex>,
        module_weights: &[HashMap<String, f64>],
    ) -> Vec<u8> {
        // Calculate total resource requirements for each module
        let _total_module_weights: Vec<u32> = module_weights
            .iter()
            .map(|mw| mw.values().sum::<f64>() as u32)
            .collect();

        // Initialize partition assignment
        let initial_part = vec![0u8; _hyprgraph.number_of_modules()];

        // TODO: Full partitioning flow with MLKWayPartMgr equivalent
        // For now, return the initial partition

        initial_part
    }

    pub fn validate_partition(
        &self,
        partition: &[u8],
        module_weights: &[HashMap<String, f64>],
    ) -> (bool, HashMap<String, f64>) {
        // Initialize resource usage tracking for each FPGA
        let mut fpga_resource_usage: Vec<HashMap<String, f64>> = Vec::new();
        for resources in &self.fpga_resources {
            let mut usage = HashMap::new();
            for resource in resources.keys() {
                usage.insert(resource.clone(), 0.0);
            }
            fpga_resource_usage.push(usage);
        }

        // Calculate resource usage
        for (module_idx, &fpga_idx) in partition.iter().enumerate() {
            let fpga_idx = fpga_idx as usize;
            if fpga_idx >= self.num_fpgas as usize {
                let mut error: HashMap<String, f64> = HashMap::new();
                error.insert("error".to_string(), fpga_idx as f64);
                return (false, error);
            }
            if let Some(module_resource_weights) = module_weights.get(module_idx) {
                for (resource, &weight) in module_resource_weights {
                    if let Some(usage) = fpga_resource_usage[fpga_idx].get_mut(resource) {
                        *usage += weight;
                    }
                }
            }
        }

        // Check resource constraints
        for (usage, capacity) in fpga_resource_usage.iter().zip(self.fpga_resources.iter()) {
            for (resource, &used) in usage {
                if let Some(&cap) = capacity.get(resource) {
                    if used > cap {
                        let mut error: HashMap<String, f64> = HashMap::new();
                        error.insert("error".to_string(), used - cap);
                        return (false, error);
                    }
                }
            }
        }

        let mut details = HashMap::new();
        details.insert("total_cost".to_string(), self.total_cost as f64);
        (true, details)
    }

    pub fn optimize_inter_fpga_connections(&self, partition: &[u8]) -> Vec<u8> {
        partition.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hypergraph::SimpleNetlist;

    fn make_resources() -> Vec<HashMap<String, f64>> {
        vec![
            HashMap::from([("lut".to_string(), 1000.0), ("ff".to_string(), 2000.0)]),
            HashMap::from([("lut".to_string(), 1200.0), ("ff".to_string(), 2400.0)]),
        ]
    }

    #[test]
    fn test_new_default() {
        let mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        assert_eq!(mgr.num_fpgas, 2);
        assert_eq!(mgr.fpga_resources.len(), 2);
    }

    #[test]
    fn test_partition_design_basic() {
        let netlist = SimpleNetlist::new(4, 2);
        let module_weights = vec![
            HashMap::from([("lut".to_string(), 200.0)]),
            HashMap::from([("lut".to_string(), 150.0)]),
            HashMap::from([("lut".to_string(), 300.0)]),
            HashMap::from([("lut".to_string(), 250.0)]),
        ];
        let mut mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        let part = mgr.partition_design(&netlist, &module_weights);
        assert_eq!(part.len(), 4);
    }

    #[test]
    fn test_validate_partition_valid() {
        let module_weights = vec![
            HashMap::from([("lut".to_string(), 200.0), ("ff".to_string(), 400.0)]),
            HashMap::from([("lut".to_string(), 150.0), ("ff".to_string(), 300.0)]),
        ];
        let mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        let partition = vec![0u8, 1];
        let (valid, details) = mgr.validate_partition(&partition, &module_weights);
        assert!(valid);
        assert!(details.contains_key("total_cost"));
    }

    #[test]
    fn test_validate_partition_invalid() {
        let module_weights = vec![
            HashMap::from([("lut".to_string(), 800.0), ("ff".to_string(), 1500.0)]),
            HashMap::from([("lut".to_string(), 500.0), ("ff".to_string(), 1000.0)]),
        ];
        let mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        let partition = vec![0u8, 0];
        let (valid, details) = mgr.validate_partition(&partition, &module_weights);
        assert!(!valid);
        assert!(details.contains_key("error"));
    }

    #[test]
    fn test_validate_partition_nonexistent_fpga() {
        let module_weights = vec![HashMap::from([("lut".to_string(), 200.0)])];
        let mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        let partition = vec![5u8];
        let (valid, details) = mgr.validate_partition(&partition, &module_weights);
        assert!(!valid);
        assert!(details.contains_key("error"));
    }

    #[test]
    fn test_optimize_returns_same() {
        let mgr = MultiFPGAPartMgr::new(2, make_resources(), 0.1);
        let partition = vec![0u8, 1, 0];
        let result = mgr.optimize_inter_fpga_connections(&partition);
        assert_eq!(result, partition);
    }
}
