use petgraph::graph::Graph;

fn main() {
    let mut graph = Graph::<&str, u32>::new();
    let origin = graph.add_node("Denver");
    let destination_1 = graph.add_node("San Diego");
    let destination_2 = graph.add_node("New York");

    graph.extend_with_edges(&[
        (origin, destination_1, 250),
        (origin, destination_2, 1099)
    ]);

    println!("{}", serde_json::to_string_pretty(&graph).unwrap());
    // println!("Hello, world!");
}
