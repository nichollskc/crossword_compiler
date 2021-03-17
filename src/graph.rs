use log::{info,warn,debug};
use std::collections::{HashSet,HashMap,VecDeque};

use crate::utils::Counter;

use thiserror::Error;

#[derive(Error,Debug)]
pub enum GraphError {
    #[error("Node not found {0}")]
    NodeNotFound(usize),

    #[error("Invalid edge in graph: {0:?}, node {1} not found")]
    InvalidEdge(Edge, usize),
}

fn sorted_vec_from_set(set: HashSet<usize>) -> Vec<usize> {
    let mut vec: Vec<usize> = set.into_iter().collect();
    vec.sort();
    vec
}

#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq,PartialOrd,Ord)]
pub struct Edge(usize, usize);

#[derive(Clone,Debug)]
struct Node {
    // Original ID given to the node
    node_id: usize,
    // Set of nodes this node is connected to
    connected_nodes: HashSet<usize>,
}

impl Node {
    /// Create a new node with no edges connected to it
    fn new(node_id: usize) -> Self {
        Node {
            node_id,
            connected_nodes: HashSet::new(),
        }
    }

    /// Add neighbour to this node. Note that the reverse edge should be (manually) added to the neighbour.
    fn add_edge(&mut self, neighbour_id: usize) {
        self.connected_nodes.insert(neighbour_id);
    }

    /// Remove one of the neighbours of this node. Note that the reverse edge should be (manually) removed from the neighbour.
    fn remove_edge(&mut self, neighbour_id: usize) {
        self.connected_nodes.remove(&neighbour_id);
    }
}

#[derive(Clone,Debug)]
pub struct Graph {
    // Node storage, which may contain dead nodes
    node_storage: Vec<Node>,
    // Hashmap of all nodes in the graph, indexed by their fixed node_id
    // The value in the hashmap is the node's index in the node_storage
    node_map: HashMap<usize, usize>,
}

impl Graph {
    /// Constructs an undirected graph from a list of edges.
    ///
    /// The graph will contain a node for each node_id included in some edge,
    /// and for each edge (a,b) passed to the function, it will contain both
    /// the edge a->b and the edge b->a.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
    /// assert!(graph.is_connected());
    /// ```
    pub fn new_from_edges(edges: Vec<(usize, usize)>) -> Self {
        let mut graph: Graph = Graph {
            node_storage: vec![],
            node_map: HashMap::new(),
        };

        graph.add_edges(edges);
        graph
    }

    /// Adds edges to the undirected graph.
    ///
    /// A node will be added for each new node_id included in an edge,
    /// and for each edge (a,b) passed to the function, the graph will now
    /// contain both the edge a->b and the edge b->a.
    /// ```
    /// let mut graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (2, 3)]);
    /// assert!(!graph.is_connected());
    /// graph.add_edges(vec![(1, 2), (3, 4)]);
    /// assert!(graph.is_connected());
    /// ```
    pub fn add_edges(&mut self, edges: Vec<(usize, usize)>) {
        for edge in edges.iter() {
            debug!("Edge {:#?}", edge);
            let (first, second) = edge;

            // Check the nodes already exist, and add them if not
            self.add_node(*first);
            self.add_node(*second);

            // Then fetch the nodes and add each as a neighbour to the other
            // Note we just added these nodes, so it should be safe to fetch them!
            self.get_node_mut(*first).expect("Only just added this node, it should exist!").add_edge(*second);
            self.get_node_mut(*second).expect("Only just added this node, it should exist!").add_edge(*first);
        }
    }

    /// Adds a disconnected node to the graph, returning true if the node was already present.
    ///
    /// ```
    /// let mut graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2)]);
    /// assert!(graph.is_connected());
    /// assert!(graph.add_node(0));
    /// assert!(!graph.add_node(3));
    /// assert!(!graph.is_connected());
    /// ```
    pub fn add_node(&mut self, node_id: usize) -> bool {
        let already_present: bool = self.node_map.contains_key(&node_id);
        if !already_present {
            debug!("Adding node {}", node_id);
            // Create node and find the index it will have
            let node: Node = Node::new(node_id);
            let node_index: usize = self.count_nodes();

            self.node_storage.push(node);
            self.node_map.insert(node_id, node_index);
        }
        already_present
    }

    /// Returns the number of nodes in the graph.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (100, 101)]);
    /// assert_eq!(graph.count_nodes(), 5);
    /// ```
    pub fn count_nodes(&self) -> usize {
        self.node_storage.len()
    }

    /// Returns the number of (undirected) edges in the graph.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (100, 101)]);
    /// assert_eq!(graph.count_edges(), 3);
    /// ```
    pub fn count_edges(&self) -> usize {
        let mut edge_count: usize = 0;
        for node in self.node_storage.iter() {
            edge_count += node.connected_nodes.len();
        }
        edge_count / 2
    }

    /// Returns true if all nodes are in one connected component, false otherwise
    ///
    /// ```
    /// let mut graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (2, 3)]);
    /// assert!(!graph.is_connected());
    /// graph.add_edges(vec![(1, 2), (3, 4)]);
    /// assert!(graph.is_connected());
    /// ```
    pub fn is_connected(&self) -> bool {
        let mut connected = true;
        if self.count_nodes() > 0 {
            let node_visit_counts = self.traverse_count_node_visits();
            for node_id in self.node_map.keys() {
                // If the node does not have any visits, the whole graph is disconnected
                if !node_visit_counts.contains_key(node_id) {
                    connected = false;
                    info!("Node never reached {}", node_id);
                }
            }
        }
        connected
    }

    /// Counts cycles in the graph, with the assumption that it is connected.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0), (2, 4), (4, 3)]);
    /// assert_eq!(graph.count_cycles(), 2);
    /// ```
    pub fn count_cycles(&self) -> usize {
        if !self.is_connected() {
            warn!("Counting cycles in a graph which is not connected - we may miss some cycles!");
        }
        self.count_edges() + 1 - self.count_nodes()
    }

    /// Returns a list of all leaves in the graph i.e. nodes connected to at most one other node.
    ///
    /// These nodes can be safely removed from the graph without increasing the number
    /// of connected components.
    ///
    /// ```
    /// // A simple cycle - no leaves
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0)]);
    /// assert_eq!(graph.find_leaves(), Vec::<usize>::new());
    ///
    /// // More complex graph with leaves
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 4), (2, 5), (3, 6)]);
    /// assert_eq!(graph.find_leaves(), vec![0, 4, 5, 6]);
    /// ```
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
        leaves.sort();
        leaves
    }

    /// Given two node IDs, split the graph into two connected components, one containing
    /// first_node and one containing second_node. Returns an error if the nodes aren't present.
    /// Also returns an error if an inconsistency of the graph is discovered during the process.
    ///
    /// The partition is deterministic, and order of nodes returned within each partition is
    /// also deterministic.
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
    /// assert_eq!(graph.partition_nodes(0, 3).unwrap(), (vec![0, 1], vec![2, 3]));
    /// ```
    pub fn partition_nodes(&self, first_node: usize, second_node: usize) -> Result<(Vec<usize>, Vec<usize>), GraphError> {
        // Set up sets to keep track of nodes visited
        let mut first_node_set: HashSet<usize> = HashSet::new();
        first_node_set.insert(first_node);
        let mut second_node_set: HashSet<usize> = HashSet::new();
        second_node_set.insert(second_node);

        // Set to keep track of edges already used
        let mut used_edges: HashSet<Edge> = HashSet::new();

        // Edges to try next - initialised with edges from first and second nodes
        // Each entry is of the form (from_first_node, edge)
        let mut edge_stack: VecDeque<(bool, Edge)> = VecDeque::new();
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
            debug!("Looking at edge {:?}, already considered {}", edge, edge_already_used);

            // Also add the reverse edge to the set of used edges
            let Edge(node_from, node_to) = edge;
            used_edges.insert(Edge(node_to, node_from));

            // Visit the node this edge points to as long as we haven't already used the edge
            if !edge_already_used {
                let is_first_visit = !first_node_set.contains(&node_to) && !second_node_set.contains(&node_to);

                // If we haven't already visited this node, add it to the relevant set
                // and add new edges to the stack
                if is_first_visit {
                    debug!("First visit to node {}", node_to);

                    // Add to the relevant set
                    if from_first {
                        first_node_set.insert(node_to);
                    } else {
                        second_node_set.insert(node_to);
                    }

                    // If this is our first visit to this node, add all edges onto the stack
                    // If we can't find the node this edge pointed to, there is an invalid edge in
                    // the graph, return an error so we are aware the graph is broken
                    for edge in self._get_edge_list(node_to).map_err(|_| GraphError::InvalidEdge(edge, node_to))? {
                        edge_stack.push_back((from_first, edge));
                    }
                }
            }
        }

        // Finally, return the partition as two sorted vectors of nodes
        Ok((sorted_vec_from_set(first_node_set), sorted_vec_from_set(second_node_set)))
    }

    /// Delete the given node and return the connected components of the graph after
    /// this deletion.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0), (0, 5)]);
    /// assert_eq!(graph.clone().components_after_deleting_node(1).unwrap(), vec![vec![0, 2, 3, 4, 5]]);
    /// assert_eq!(graph.clone().components_after_deleting_node(0).unwrap(), vec![vec![1, 2], vec![3, 4], vec![5]]);
    /// ```
    pub fn components_after_deleting_node(&mut self, node_id: usize) -> Result<Vec<Vec<usize>>,GraphError> {
        self.delete_node(node_id)?;
        self.get_connected_components()
    }

    /// Return the connected components of the graph as a list of lists of nodes.
    /// Order of components, and order of nodes within the components is deterministic.
    /// Returns an error if an inconsistency in the graph is found.
    ///
    /// ```
    /// let graph = crossword::graph::Graph::new_from_edges(vec![(0, 1), (2, 3)]);
    /// assert_eq!(graph.get_connected_components().unwrap(), vec![vec![0, 1], vec![2, 3]]);
    /// ```
    pub fn get_connected_components(&self) -> Result<Vec<Vec<usize>>,GraphError> {
        let mut components: Vec<Vec<usize>> = vec![];
        let mut nodes_visited: HashSet<usize> = HashSet::new();

        while let Some(unvisited_node_id) = self.first_node_not_in_set(&nodes_visited) {
            let node_visit_counts = self.traverse_count_node_visits_from_node(unvisited_node_id)?;
            let mut component: Vec<usize> = vec![];
            for node_id in node_visit_counts.keys() {
                component.push(*node_id);
                nodes_visited.insert(*node_id);
            }
            component.sort();
            components.push(component);
        }
        Ok(components)
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

    // Starting from an arbitrary node, performs a search through all nodes
    // that can be reached from that node and counts the number of visits made
    // to each node in the graph.
    fn traverse_count_node_visits(&self) -> HashMap<usize, usize> {
        if self.node_storage.len() > 0 {
            // Pick the first node as a starting point, and count number of visits after
            // traversal
            let node_id = self.node_storage[0].node_id;
            self.traverse_count_node_visits_from_node(node_id).expect("Node id should be present - selected as first in list")
        } else {
            HashMap::new()
        }
    }

    // Starting from the given node, performs a search through all nodes
    // that can be reached from that node and counts the number of visits made
    // to each node in the graph. Returns an error if the node is not in the graph.
    fn traverse_count_node_visits_from_node(&self, node_id: usize) -> Result<HashMap<usize, usize>, GraphError> {
        let mut node_visit_counts: Counter<usize> = Counter::new();
        node_visit_counts.increment(node_id);

        let mut used_edges: HashSet<Edge> = HashSet::new();
        let mut edge_stack: Vec<Edge> = self._get_edge_list(node_id)?;
        debug!("Edge stack to start {:#?}", edge_stack);

        while let Some(edge) = edge_stack.pop() {
            // Only traverse edge if we haven't already used it
            let edge_already_used: bool = !used_edges.insert(edge);

            // Also add the reverse edge to the set of used edges
            let Edge(first, second) = edge;
            used_edges.insert(Edge(second, first));
            debug!("Looking at edge {:#?}, already considered {}", edge, edge_already_used);
            if !edge_already_used {
                // Visit the node this edge points to
                let next_node = edge.1;

                // Update the count for this visit to node next_node
                let already_visited = node_visit_counts.increment(next_node);
                if !already_visited {
                    debug!("First visit to node {}", next_node);

                    // If this is our first visit, add all edges onto the stack
                    match self._get_edge_list(next_node) {
                        Ok(mut new_nodes_to_visit) => edge_stack.append(&mut new_nodes_to_visit),
                        Err(error) => panic!("Graph inconsistent - node should be present as we found it through an edge {:?}. Error: {:?}", edge, error),
                    };
                }
            }
        }
        Ok(node_visit_counts.into_hashmap())
    }

    // Attempt to find a list of all the edges starting from this node. Return an error
    // if the node doesn't exist.
    //
    // Order of the edges is deterministic.
    fn _get_edge_list(&self, node_id: usize) -> Result<Vec<Edge>, GraphError> {
        let node = self.get_node(node_id)?;
        let mut edges: Vec<Edge> = vec![];
        for neighbour_id in node.connected_nodes.iter() {
            edges.push(Edge(node_id, *neighbour_id));
        }
        edges.sort();
        Ok(edges)
    }

    // Return the node with the smallest node_id which is not contained in the set forbidden_nodes
    fn first_node_not_in_set(&self, forbidden_nodes: &HashSet<usize>) -> Option<usize> {
        let mut allowed_nodes: Vec<usize> = self.node_map.keys().filter(|n| !forbidden_nodes.contains(n)).cloned().collect();
        allowed_nodes.sort();
        allowed_nodes.reverse();
        allowed_nodes.pop()
    }

    // Delete the node, and all edges in the graph involving the node. Return
    // true if the node was deleted, false if it wasn't found.
    fn delete_node(&mut self, node_id: usize) -> Result<bool,GraphError> {
        let connected_nodes = match self.get_node_mut(node_id) {
            Ok(node) => node.connected_nodes.drain().collect(),
            Err(_) => HashSet::new(),
        };

        for neighbour_id in connected_nodes.iter() {
            let neighbour = self.get_node_mut(*neighbour_id)
                .map_err(|_| GraphError::InvalidEdge(Edge(node_id, *neighbour_id), *neighbour_id))?;
            neighbour.remove_edge(node_id);
        }

        let was_deleted = self.shift_node_storage_after_removal(node_id);
        Ok(was_deleted)
    }

    // Delete a node from node storage, and from the node map, and update the
    // node_storage and node_map to reflect this
    fn shift_node_storage_after_removal(&mut self, node_id: usize) -> bool {
        let mut was_deleted = false;
        let index_in_storage = self.node_map.remove(&node_id);
        if let Some(index) = index_in_storage {
            // Remove the node, shifting all to the left
            self.node_storage.remove(index);

            let mut i = index;
            while i < self.node_storage.len() {
                let node = &self.node_storage[i];
                self.node_map.insert(node.node_id, i);
                i += 1;
            }
            was_deleted = true;
        }
        was_deleted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_graph(graph: &Graph, expected_nodes: usize, expected_edges: usize) {
        debug!("Result is {:#?}", graph);
        assert_eq!(graph.count_nodes(), expected_nodes);
        assert_eq!(graph.count_edges(), expected_edges);
    }

    #[test]
    fn build_graph_complex() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (2, 0)]);
        check_graph(&graph, 4, 4);

        let graph = Graph::new_from_edges(vec![(0, 1), (0, 1), (0, 1), (1, 0)]);
        check_graph(&graph, 2, 1);
    }

    #[test]
    fn build_graph_basic() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        check_graph(&graph, 4, 3);

        let graph = Graph::new_from_edges(vec![(10, 11), (11, 20), (20, 5)]);
        check_graph(&graph, 4, 3);
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
    fn test_partition_nodes() {
        crate::logging::init_logger(true);
        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_nodes(0, 3).unwrap(), (vec![0, 1], vec![2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_nodes(3, 0).unwrap(), (vec![2, 3], vec![0, 1]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_nodes(0, 1).unwrap(), (vec![0], vec![1, 2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3)]);
        assert_eq!(graph.partition_nodes(1, 0).unwrap(), (vec![1, 2, 3], vec![0]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(graph.partition_nodes(0, 3).unwrap(), (vec![0, 1], vec![2, 3]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 3), (3, 0)]);
        assert_eq!(graph.partition_nodes(0, 2).unwrap(), (vec![0, 1, 3], vec![2]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        assert_eq!(graph.partition_nodes(0, 3).unwrap(), (vec![0, 1, 2], vec![3, 4, 5]));

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (5, 3), (3, 4), (4, 5)]);
        assert_eq!(graph.partition_nodes(3, 0).unwrap(), (vec![3, 4, 5], vec![0, 1, 2]));
    }

    #[test]
    fn test_partition_nodes_robust() {
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
                let (first, second) = graph.partition_nodes(i, j).unwrap();

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
        assert_eq!(graph.clone().components_after_deleting_node(1).unwrap(), vec![vec![0, 2, 3, 4]]);
        assert_eq!(graph.clone().components_after_deleting_node(0).unwrap(), vec![vec![1, 2], vec![3, 4]]);

        let graph = Graph::new_from_edges(vec![(0, 1), (1, 2), (2, 0), (0, 3), (3, 4), (4, 0), (0, 5)]);
        assert_eq!(graph.clone().components_after_deleting_node(1).unwrap(), vec![vec![0, 2, 3, 4, 5]]);
        assert_eq!(graph.clone().components_after_deleting_node(0).unwrap(), vec![vec![1, 2], vec![3, 4], vec![5]]);

        let mut graph = Graph::new_from_edges(vec![(0, 1)]);
        println!("{:#?}", graph);
        assert_eq!(graph.components_after_deleting_node(0).unwrap(), vec![vec![1]]);
        check_graph(&graph, 1, 0);
        println!("{:#?}", graph);
        assert_eq!(graph.components_after_deleting_node(1).unwrap(), Vec::<Vec<usize>>::new());
        check_graph(&graph, 0, 0);
    }
}
