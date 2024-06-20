/// Represents movement information including the net, vertex, and partition details.
#[derive(Debug, Copy, Clone)]
pub struct MoveInfo<Node> {
    /// The identifier for the net involved in the move.
    pub net: Node,
    /// The vertex being moved.
    pub v: Node,
    /// The original partition from which the vertex is moving.
    pub from_part: u8,
    /// The target partition to which the vertex is moving.
    pub to_part: u8,
}

/// Represents simplified movement information focusing on vertex and partition changes.
#[derive(Debug, Copy, Clone)]
pub struct MoveInfoV<Node> {
    /// The vertex being moved.
    pub v: Node,
    /// The original partition from which the vertex is moving.
    pub from_part: u8,
    /// The target partition to which the vertex is moving.
    pub to_part: u8,
    // Note: The commented-out 'node_t v;' line from C++ is redundant and thus not included.
}

#[cfg(test)]
mod tests {
    extern crate petgraph;
    use super::*;
    use petgraph::prelude::*;

    #[test]
    fn test_move_info_creation_and_access() {
        // Create an empty graph just to obtain a NodeIndex for the test
        let mut graph: Graph<(), (), Directed> = Graph::new();
        let node_a = graph.add_node(());

        let move_info = MoveInfo {
            net: node_a,
            v: node_a, // Using the same node for simplicity
            from_part: 0,
            to_part: 1,
        };

        assert_eq!(move_info.net.index(), 0);
        assert_eq!(move_info.v.index(), 0);
        assert_eq!(move_info.from_part, 0);
        assert_eq!(move_info.to_part, 1);
    }

    #[test]
    fn test_move_info_v_creation_and_access() {
        // Reusing the graph from the previous test
        let move_info_v = MoveInfoV {
            v: 0, // Directly creating a NodeIndex for simplicity
            from_part: 0,
            to_part: 1,
        };

        assert_eq!(move_info_v.v, 0);
        assert_eq!(move_info_v.from_part, 0);
        assert_eq!(move_info_v.to_part, 1);
    }
}
