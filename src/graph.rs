use log::{info,warn,debug,error};
use std::collections::{HashSet,HashMap};

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
pub struct Graph {
    // Node storage
    node_storage: Vec<Node>,
    // Hashmap of all nodes in the graph, indexed by their fixed node_id
    // The value in the hashmap is the node's index in the node_storage
    node_map: HashMap<usize, usize>,
    num_nodes: usize,
}

impl Graph {
    pub fn new_from_edges(edges: Vec<(usize, usize)>) -> Self {
        let mut graph: Graph = Graph {
            num_nodes: 0,
            node_storage: vec![],
            node_map: HashMap::new(),
        };

        graph.add_edges(edges);
        graph
    }

    pub fn add_edges(&mut self, edges: Vec<(usize, usize)>) {
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

    pub fn add_node(&mut self, node_id: usize) -> bool {
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
        match self.node_map.get(&node_id) {
            Some(index) => Some(&mut self.node_storage[*index]),
            None => None,
        }
    }

    fn get_node(&self, node_id: usize) -> Option<&Node> {
        match self.node_map.get(&node_id) {
            Some(index) => Some(& self.node_storage[*index]),
            None => None,
        }
    }

    pub fn count_edges(&self) -> usize {
        let mut edge_count: usize = 0;
        for node in self.node_storage.iter() {
            edge_count += node.connected_nodes.len();
        }
        edge_count / 2
    }

    fn traverse_count_node_visits(&self) -> HashMap<usize, usize> {
        let mut node_visits: HashMap<usize, usize> = HashMap::new();
        let node_id = self.node_storage[0].node_id;
        node_visits.insert(node_id, 1);

        let mut used_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut edge_stack: Vec<(usize, usize)> = self._get_edge_list(node_id);
        println!("Edge stack to start {:#?}", edge_stack);

        while let Some(edge) = edge_stack.pop() {
            // Only traverse edge if we haven't already used it
            let edge_already_used: bool = !used_edges.insert(edge);

            // Also add the reverse edge to the set of used edges
            let (first, second) = edge;
            used_edges.insert((second, first));
            println!("Looking at edge {:#?}, already considered {}", edge, edge_already_used);
            if !edge_already_used {
                // Visit the node this edge points to
                let next_node = edge.1;

                // Update the count
                match node_visits.get_mut(&next_node) {
                    Some(visit_count) => *visit_count += 1,
                    None => {
                        println!("First visit to node {}", next_node);
                        node_visits.insert(next_node, 1);

                        // If this is our first visit, add all edges onto the stack
                        edge_stack.append(&mut self._get_edge_list(next_node));
                    },
                }
            }
        }
        node_visits
    }

    /// Returns true if all nodes are in one connected component, false otherwise
    pub fn is_connected(&self) -> bool {
        let mut connected = true;
        let node_visits = self.traverse_count_node_visits();
        for node_id in self.node_map.keys() {
            if !node_visits.contains_key(node_id) {
                connected = false;
                println!("Node never reached {}", node_id);
            }
        }
        connected
    }

    /// Counts cycles in the graph, with the assumption that it is connected
    pub fn count_cycles(&self) -> usize {
        self.count_edges() + 1 - self.num_nodes
    }

    fn _get_edge_list(&self, node_id: usize) -> Vec<(usize, usize)> {
        match self.get_node(node_id) {
            Some(node) => {
                let mut edges: Vec<(usize, usize)> = vec![];
                for neighbour_id in node.connected_nodes.iter() {
                    edges.push((node_id, *neighbour_id));
                }
                edges
            },
            None => vec![],
        }
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

    #[test]
    fn traverse_graph() {
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        println!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 0);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        println!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 1);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)]);
        println!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 2);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        println!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert!(!graph.is_connected());
    }
}
