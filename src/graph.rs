use log::{info,warn,debug,error};
use std::collections::{HashSet,HashMap};

pub mod graph {
    pub struct Graph;
}

#[derive(Debug)]
struct Node {
    // Original ID given to the node
    node_id: usize,
    // Set of nodes this node is connected to
    connected_nodes: HashSet<usize>, 
}

impl Node {
    fn new(node_id: usize) -> Self {
        Node {
            node_id,
            connected_nodes: HashSet::new(),
        }
    }

    fn add_edge(&mut self, neighbour_id: usize) {
        self.connected_nodes.insert(neighbour_id);
    }

    fn remove_edge(&mut self, neighbour_id: usize) {
        self.connected_nodes.remove(&neighbour_id);
    }
}

#[derive(Debug)]
struct Graph {
    // Node storage
    node_storage: Vec<Node>,
    // Hashmap of all nodes in the graph, indexed by their fixed node_id
    // The value in the hashmap is the node's index in the node_storage
    node_map: HashMap<usize, usize>,
    num_nodes: usize,
}

impl Graph {
    fn new_from_edges(edges: Vec<(usize, usize)>) -> Self {
        let mut graph: Graph = Graph {
            num_nodes: 0,
            node_storage: vec![],
            node_map: HashMap::new(),
        };

        graph.add_edges(edges);
        graph
    }

    fn add_edges(&mut self, edges: Vec<(usize, usize)>) {
        for edge in edges.iter() {
            debug!("Edge {:#?}", edge);
            let (first, second) = edge;

            // Check the nodes already exist, and add them if not
            self.add_node(*first);
            self.add_node(*second);

            // Then fetch the nodes and add each as a neighbour to the other
            self.get_node_mut(*first).unwrap().add_edge(*second);
            self.get_node_mut(*second).unwrap().add_edge(*first);
        }
    }

    fn add_node(&mut self, node_id: usize) -> bool {
        let already_present: bool = self.node_map.contains_key(&node_id);
        if !already_present {
            debug!("Adding node {}", node_id);
            // Create node and find the index it will have
            let node: Node = Node::new(node_id);
            let node_index: usize = self.num_nodes;

            self.num_nodes += 1;
            self.node_storage.push(node);
            self.node_map.insert(node_id, node_index);
        }
        already_present
    }
    
    fn get_node_mut(&mut self, node_id: usize) -> Option<&mut Node> {
        let node_index: Option<&usize> = self.node_map.get(&node_id);
        match node_index {
            Some(index) => Some(&mut self.node_storage[*index]),
            None => None,
        }
    }

    fn count_edges(&self) -> usize {
        let mut edge_count: usize = 0;
        for node in self.node_storage.iter() {
            edge_count += node.connected_nodes.len();
        }
        edge_count / 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_graph(graph: Graph, expected_nodes: usize, expected_edges: usize) {
        println!("Result is {:#?}", graph);
        assert_eq!(graph.num_nodes, expected_nodes);
        assert_eq!(graph.node_storage.len(), expected_nodes);
        assert_eq!(graph.count_edges(), expected_edges);
    }

    #[test]
    fn build_graph_complex() {
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (2, 0)]);
        check_graph(graph, 4, 4);

        let graph = Graph::new_from_edges(vec![(0, 1), (0, 1), (0, 1), (1, 0)]);
        check_graph(graph, 2, 1);
    }

    #[test]
    fn build_graph_basic() {
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        check_graph(graph, 4, 3);

        let graph = Graph::new_from_edges(vec![(10, 11), (11, 20), (20, 5)]);
        check_graph(graph, 4, 3);
    }
}
