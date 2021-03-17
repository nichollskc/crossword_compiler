use log::{warn,debug};
use std::collections::{HashSet,HashMap,VecDeque};

use thiserror::Error;

#[derive(Error,Debug)]
pub enum GraphError {
    #[error("Node not found {0}")]
    NodeNotFound(usize),

    #[error("Invalid edge in graph: {0:?}, node {1} not found")]
    InvalidEdge((usize, usize), usize),
}

#[derive(Clone,Debug)]
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

#[derive(Clone,Debug)]
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
            // Note we just added these nodes, so it should be safe to fetch them!
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

    fn get_node_mut(&mut self, node_id: usize) -> Result<&mut Node, GraphError> {
        match self.node_map.get(&node_id) {
            Some(index) => Ok(&mut self.node_storage[*index]),
            None => Err(GraphError::NodeNotFound(node_id)),
        }
    }

    fn get_node(&self, node_id: usize) -> Result<&Node, GraphError> {
        match self.node_map.get(&node_id) {
            Some(index) => Ok(&self.node_storage[*index]),
            None => Err(GraphError::NodeNotFound(node_id)),
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
        if self.node_storage.len() > 0 {
            let node_id = self.node_storage[0].node_id;
            self.traverse_count_node_visits_from_node(node_id).expect("Node id should be present - selected as first in list")
        } else {
            HashMap::new()
        }
    }

    fn traverse_count_node_visits_from_node(&self, node_id: usize) -> Result<HashMap<usize, usize>, GraphError> {
        let mut node_visits: HashMap<usize, usize> = HashMap::new();
        node_visits.insert(node_id, 1);

        let mut used_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut edge_stack: Vec<(usize, usize)> = self._get_edge_list(node_id)?;
        debug!("Edge stack to start {:#?}", edge_stack);

        while let Some(edge) = edge_stack.pop() {
            // Only traverse edge if we haven't already used it
            let edge_already_used: bool = !used_edges.insert(edge);

            // Also add the reverse edge to the set of used edges
            let (first, second) = edge;
            used_edges.insert((second, first));
            debug!("Looking at edge {:#?}, already considered {}", edge, edge_already_used);
            if !edge_already_used {
                // Visit the node this edge points to
                let next_node = edge.1;

                // Update the count
                match node_visits.get_mut(&next_node) {
                    Some(visit_count) => *visit_count += 1,
                    None => {
                        debug!("First visit to node {}", next_node);
                        node_visits.insert(next_node, 1);

                        // If this is our first visit, add all edges onto the stack
                        match self._get_edge_list(next_node) {
                            Ok(mut new_nodes_to_visit) => edge_stack.append(&mut new_nodes_to_visit),
                            Err(error) => panic!("Graph inconsistent - node should be present as we found it in an edge {:?}. Error: {:?}", edge, error),
                        };
                    },
                }
            }
        }
        Ok(node_visits)
    }

    /// Returns true if all nodes are in one connected component, false otherwise
    pub fn is_connected(&self) -> bool {
        let mut connected = true;
        if self.num_nodes > 0 {
            let node_visits = self.traverse_count_node_visits();
            for node_id in self.node_map.keys() {
                if !node_visits.contains_key(node_id) {
                    connected = false;
                    warn!("Node never reached {}", node_id);
                }
            }
        }
        connected
    }

    /// Counts cycles in the graph, with the assumption that it is connected
    pub fn count_cycles(&self) -> usize {
        self.count_edges() + 1 - self.num_nodes
    }

    pub fn find_leaves(&self) -> Vec<usize> {
        let mut leaves = vec![];
        for node_id in self.node_map.keys() {
            match self.get_node(*node_id) {
                Ok(node) => {
                    // A leaf is a node which has only one edge
                    if node.connected_nodes.len() <= 1 {
                        leaves.push(*node_id);
                    }
                },
                Err(error) => panic!("Graph found to be inconsistent in search for leaves: {:?}", error),
            };
        }
        leaves
    }

    fn _get_edge_list(&self, node_id: usize) -> Result<Vec<(usize, usize)>, GraphError> {
        let node = self.get_node(node_id)?;
        let mut edges: Vec<(usize, usize)> = vec![];
        for neighbour_id in node.connected_nodes.iter() {
            edges.push((node_id, *neighbour_id));
        }
        edges.sort();
        Ok(edges)
    }

    /// Given two node IDs, split the graph into two connected components, one containing
    /// first_node and one containing second_node.
    pub fn partition_graph(&self, first_node: usize, second_node: usize) -> Result<(Vec<usize>, Vec<usize>), GraphError> {
        let mut node_visits: HashMap<usize, usize> = HashMap::new();
        node_visits.insert(first_node, 1);
        node_visits.insert(second_node, 1);

        let mut first_node_set: HashSet<usize> = HashSet::new();
        first_node_set.insert(first_node);
        let mut second_node_set: HashSet<usize> = HashSet::new();
        second_node_set.insert(second_node);

        let mut used_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut edge_stack: VecDeque<(bool, (usize, usize))> = VecDeque::new();

        for edge in self._get_edge_list(first_node)? {
            edge_stack.push_back((true, edge));
        }
        for edge in self._get_edge_list(second_node)? {
            edge_stack.push_back((false, edge));
        }
        debug!("Edge stack to start {:?}", edge_stack);

        while let Some((from_first, edge)) = edge_stack.pop_front() {
            debug!("Node sets: {:?}\n{:?}", first_node_set, second_node_set);
            // Only traverse edge if we haven't already used it
            let edge_already_used: bool = !used_edges.insert(edge);

            // Also add the reverse edge to the set of used edges
            let (node_from, node_to) = edge;
            used_edges.insert((node_to, node_from));
            debug!("Looking at edge {:?}, already considered {}", edge, edge_already_used);
            if !edge_already_used {
                // Visit the node this edge points to

                let is_first_visit = if from_first {
                    !second_node_set.contains(&node_to) && first_node_set.insert(node_to)
                } else {
                    !first_node_set.contains(&node_to) && second_node_set.insert(node_to)
                };

                if is_first_visit {
                    debug!("First visit to node {}", node_to);
                    // If this is our first visit, add all edges onto the stack
                    for edge in self._get_edge_list(node_to).map_err(|_| GraphError::InvalidEdge(edge, node_to))? {
                        edge_stack.push_back((from_first, edge));
                    }
                }
            }
        }
        let mut first_node_vec: Vec<usize> = first_node_set.into_iter().collect();
        first_node_vec.sort();
        let mut second_node_vec: Vec<usize> = second_node_set.into_iter().collect();
        second_node_vec.sort();
        Ok((first_node_vec, second_node_vec))
    }

    fn first_node_not_in_set(&self, forbidden_nodes: &HashSet<usize>) -> Option<usize> {
        let mut allowed_nodes: Vec<usize> = self.node_map.keys().filter(|n| !forbidden_nodes.contains(n)).cloned().collect();
        allowed_nodes.sort();
        allowed_nodes.reverse();
        allowed_nodes.pop()
    }

    pub fn components_after_deleting_node(&mut self, node_id: usize) -> Vec<Vec<usize>> {
        let connected_nodes = self.get_node(node_id).unwrap().connected_nodes.clone();
        for neighbour_id in connected_nodes.iter() {
            let neighbour = self.get_node_mut(*neighbour_id).unwrap();
            neighbour.remove_edge(node_id);
        }
        self.get_node_mut(node_id).unwrap().connected_nodes = HashSet::new();

        let mut components: Vec<Vec<usize>> = vec![];
        let mut nodes_visited: HashSet<usize> = HashSet::new();
        nodes_visited.insert(node_id);

        while let Some(unvisited_node_id) = self.first_node_not_in_set(&nodes_visited) {
            match self.traverse_count_node_visits_from_node(unvisited_node_id) {
                Ok(node_visits) => {
                    let mut component: Vec<usize> = vec![];
                    for node_id in node_visits.keys() {
                        component.push(*node_id);
                        nodes_visited.insert(*node_id);
                    }
                    component.sort();
                    components.push(component);
                },
                Err(error) => panic!("Graph found to be inconsistent when partitioning after deleting node: {:?}", error),
            };
        }
        components
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_graph(graph: Graph, expected_nodes: usize, expected_edges: usize) {
        debug!("Result is {:#?}", graph);
        assert_eq!(graph.num_nodes, expected_nodes);
        assert_eq!(graph.node_storage.len(), expected_nodes);
        assert_eq!(graph.count_edges(), expected_edges);
    }

    #[test]
    fn build_graph_complex() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (2, 0)]);
        check_graph(graph, 4, 4);

        let graph = Graph::new_from_edges(vec![(0, 1), (0, 1), (0, 1), (1, 0)]);
        check_graph(graph, 2, 1);
    }

    #[test]
    fn build_graph_basic() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        check_graph(graph, 4, 3);

        let graph = Graph::new_from_edges(vec![(10, 11), (11, 20), (20, 5)]);
        check_graph(graph, 4, 3);
    }

    #[test]
    fn traverse_graph() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        debug!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 0);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        debug!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 1);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)]);
        debug!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert_eq!(graph.count_cycles(), 2);
        assert!(graph.is_connected());

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        debug!("Traversal {:#?}", graph.traverse_count_node_visits());
        assert!(!graph.is_connected());
    }

    #[test]
    fn test_partition_graph() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_graph(0, 3).unwrap(), (vec![0, 1], vec![2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_graph(3, 0).unwrap(), (vec![2, 3], vec![0, 1]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_graph(0, 1).unwrap(), (vec![0], vec![1, 2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_graph(1, 0).unwrap(), (vec![1, 2, 3], vec![0]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(graph.partition_graph(0, 3).unwrap(), (vec![0, 1], vec![2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(graph.partition_graph(0, 2).unwrap(), (vec![0, 1, 3], vec![2]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        assert_eq!(graph.partition_graph(0, 3).unwrap(), (vec![0, 1, 2], vec![3, 4, 5]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        assert_eq!(graph.partition_graph(3, 0).unwrap(), (vec![3, 4, 5], vec![0, 1, 2]));
    }

    #[test]
    fn test_partition_graph_robust() {
        crate::logging::init_logger(true);
        let mut edges: Vec<(usize, usize)> = vec![];
        for i in 0..20 {
            edges.push((i, i+1));
            edges.push((i, (3*i + i^2 + 1).rem_euclid(21)));
        }
        edges.sort();
        let graph = Graph::new_from_edges(edges.clone());
        assert!(graph.is_connected());
        for i in 0..21 {
            for j in (i+1)..21 {
                // For each pair of indices, split the graph
                let (first, second) = graph.partition_graph(i, j).unwrap();

                // Create hashmap versions for convenience
                let first_hash: HashSet<usize> = first.iter().cloned().collect();
                let second_hash: HashSet<usize> = second.iter().cloned().collect();

                // Check that each hash contains its respective starting node
                assert!(first_hash.contains(&i));
                assert!(second_hash.contains(&j));

                // Check the intersection is empty
                let intersection: HashSet<usize> = first_hash.intersection(&second_hash).cloned().collect();
                assert!(intersection.is_empty(), "Expected empty intersection, found {:?}", intersection);

                // Check that every node is contained in one of the sets
                for k in 0..21 {
                    assert!(first_hash.contains(&k) || second_hash.contains(&k));
                }

                // Check the graphs restricted to just one set are both connected
                let first_edges: Vec<(usize, usize)> = edges.iter().filter(|e| first_hash.contains(&e.0)
                        && first_hash.contains(&e.1)).cloned().collect();
                let second_edges: Vec<(usize, usize)> = edges.iter().filter(|e| second_hash.contains(&e.0)
                        && second_hash.contains(&e.1)).cloned().collect();

                let first_graph = Graph::new_from_edges(first_edges);
                assert!(first_graph.is_connected());
                let second_graph = Graph::new_from_edges(second_edges);
                assert!(second_graph.is_connected());
            }
        }
    }

    #[test]
    fn test_components_after_node_removal() {
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0)]);
        assert_eq!(graph.clone().components_after_deleting_node(1), vec![vec![0, 2, 3, 4]]);
        assert_eq!(graph.clone().components_after_deleting_node(0), vec![vec![1, 2], vec![3, 4]]);

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0), (0, 5)]);
        assert_eq!(graph.clone().components_after_deleting_node(1), vec![vec![0, 2, 3, 4, 5]]);
        assert_eq!(graph.clone().components_after_deleting_node(0), vec![vec![1, 2], vec![3, 4], vec![5]]);

        let mut graph = Graph::new_from_edges(vec![(0, 1)]);
        println!("{:#?}", graph);
        assert_eq!(graph.components_after_deleting_node(0), vec![vec![1]]);
        println!("{:#?}", graph);
        // This doesn't happen - the node 0 is still present, just not connected to anything
        //assert_eq!(graph.components_after_deleting_node(1), Vec::<Vec<usize>>::new());
    }
}
