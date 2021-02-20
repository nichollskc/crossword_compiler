use std::collections::{HashSet,HashMap};

pub mod graph {
    pub struct Graph;
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_graph() {
        Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
    }
}
