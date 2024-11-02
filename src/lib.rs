pub mod hyperedge;
pub mod hypergraph_traits;

// Best hash for rust
use ahash::{AHashMap, AHashSet, RandomState};

use std::{
    cell::{Ref, RefCell},
    hash::{BuildHasher, Hash, Hasher},
    rc::Rc,
};

use hyperedge::Hyperedge;

// Seeds for computing the hash value for a hyperedge, ie its EdgeID.
const SEED1: u64 = 0x243F6A8885A308D3;
const SEED2: u64 = 0x13198A2E03707344;
const SEED3: u64 = 0xA4093822299F31D0;
const SEED4: u64 = 0x082EFA98EC4E6C89;

// Defined data type
type Node = i64;
type EdgeID = u64;

type IterEdges<'a> = std::collections::hash_map::Values<'a, u64, Hyperedge>;
type IterNodes<'a> = std::collections::hash_map::Keys<'a, Node, AHashSet<EdgeID>>;

/// Core struct to represent hypergraphs.  
///
/// A hypergraph is a generalization of a graph in which an edge can join any number of vertices (see [Hypergraph](https://en.wikipedia.org/wiki/Hypergraph)).  
pub struct Hypergraph {
    weighted: bool,

    incidence_list: AHashMap<Node, AHashSet<EdgeID>>,

    edge_list: AHashMap<EdgeID, Hyperedge>,
}

impl Hypergraph {
    /*
    ===============================================================================
    |                               PUBLIC API                                    |
    ===============================================================================
    */

    /// Creates a new, empty `Hypergraph`.  
    /// This is the default constructor.
    ///
    /// # Parameters
    /// - `weighted`: `bool` - Specifies whether the hypergraph is weighted (`true`), or nor (`false`).
    ///
    /// # Returns
    /// - `Self` - A new instance of `Hypergraph`.
    pub fn new(weighted: bool) -> Self {
        Self {
            weighted,
            incidence_list: AHashMap::new(),
            edge_list: AHashMap::new(),
        }
    }

    /// Creates an unweighted `Hypergraph` from a list of hyperedges.  
    ///
    /// For every duplicate in `_edge_list` there will be only an hyperedge.  
    ///
    /// # Parameters
    /// - `_edge_list`: (`&[Vec<i64>]`) - List of hyperedges, each represented as a vector of nodes.
    ///
    /// # Returns
    /// - `Self` - A new instance of `Hypergraph`.
    pub fn from(_edge_list: &[Vec<i64>]) -> Self {
        let mut result = Self::new(false);

        for edge in _edge_list.iter() {
            let edge_id = Self::compute_edge_id(edge);

            if !result.edge_list.contains_key(&edge_id) {
                Self::compute_add_edge(&mut result, edge, 0_f64);
            }
        }
        result
    }

    /// Creates a weighted `Hypergraph` from a list of hyperedges.
    ///
    /// Let `n`, `m` be the length of `_edge_list` and `weights` respectively. Consider this three cases:   
    /// - `n` > `m`: then the first m hyperedges will receive its corresponding weight, and the last `n-m` will be   
    /// initialized with 0 as their weight;
    /// - `n` = `m`: then every hyperedge will receive its corresponding weight;
    /// - `n` < `m`: same as above; there will simply be some weights which will not be assigned.  
    ///
    /// If `_edge_list` contains duplicates, the considered hyperedge, with its weight, will be the first encountered in the list.
    /// # Parameters
    /// - `_edge_list`: `&[Vec<i64>]` - List of hyperedges.
    /// - `weights`: `&[f64]` - Weights for the hyperedges.
    ///
    /// # Returns
    /// - `Self` - A new instance of `Hypergraph`.
    pub fn from_weighted(_edge_list: &[Vec<i64>], weights: &[f64]) -> Self {
        let mut result = Self::new(true);

        let mut index_weigth = 0 as usize;
        let mut next;

        for edge in _edge_list.iter() {
            // This index increases indipendently
            if index_weigth + 1 < weights.len() {
                next = weights[index_weigth];
                index_weigth += 1;
            } else {
                next = 0_f64;
            }

            Self::compute_add_edge(&mut result, edge, next);
        }
        result
    }

    /// Returns the number of nodes in the hypergraph.
    ///
    /// # Returns
    /// `usize` - Order of the hypergraph.
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn get_num_nodes(&self) -> usize {
        self.incidence_list.len()
    }

    /// Returns the number of hyperedges in the hypergraph.
    ///
    /// # Returns
    /// - `usize` - Size of the hypergraph.
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn get_num_edges(&self) -> usize {
        self.edge_list.len()
    }

    /// Returns the number of hyperedges with a specific order.
    ///
    /// # Parameters
    /// - `order` : `usize` - Order of interest.
    ///
    /// # Returns
    /// - `usize` - The number of hyperedges.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` denotes the number of hyperedges of the hypergraph.
    pub fn get_num_edges_order(&self, order: usize) -> usize {
        let mut res = 0;
        for (_, edge) in self.edge_list.iter() {
            if (*edge.nodes).borrow().len() == order {
                res += 1;
            }
        }
        res
    }

    /// Returns the weight of a specific hyperedge.
    ///
    /// # Parameters
    /// - 'edge' : `&Vec<Node>` - The Hyperedge.
    ///
    /// # Returns
    /// - `Option<f64>` - `Some` weight of the hyperedge. Returns `None` if the hyperedge is not in the hypergraph.
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn get_weigth(&self, edge: &Vec<Node>) -> Option<f64> {
        let edge_id = Self::compute_edge_id(edge);

        match self.edge_list.get(&edge_id) {
            Some(edge) => Some((*edge).weight),
            _ => None,
        }
    }

    /// Sets the weight of a specific hyperedge.
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge for which the weight has to be modified.
    /// - `new_weight` : `f64` - The new weight for the hyperedge.
    ///
    /// # Returns
    /// - `Result<f64, ()>` : `Ok(f64)`, containing the previous weight of the hyperedge, if it exists. Returns `Err(())` if the   
    /// specified hyperedge is not found.
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn set_weight(&mut self, edge: &Vec<Node>, new_weight: f64) -> Result<f64, ()> {
        let edge_id = Self::compute_edge_id(edge);

        match self.edge_list.get_mut(&edge_id) {
            Some(edge) => {
                let prev = edge.weight;
                edge.set_weight(new_weight);
                Ok(prev)
            }
            _ => Err(()),
        }
    }

    /// Returns the weights of all the hyperedges.
    ///
    /// The returned list may contain duplicates of the weights.
    ///
    /// # Returns
    /// - `Option<Vec<f64>>` - `Some` list with the weights if there are hyperedges; returns `None` if the hypergraph has no hyperedges.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` is the number of hyperedges of the hypergraph.
    pub fn get_weights(&self) -> Option<Vec<f64>> {
        if self.edge_list.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            // O(m)
            self.edge_list.values().for_each(|hyperedge| {
                res.push(hyperedge.weight);
            });
            Some(res)
        }
    }

    /// Returns the weights of the hyperedges with a specified order.
    ///
    /// The returned list may contain duplicates of the weights.
    ///
    /// # Parameters
    /// - `order` : `usize` - The order of interest.
    ///
    /// # Returns
    /// - `Option<Vec<f64>>` - `Some` list with the weights of hyperedges with the given order; `None` if no such hyperedges exist.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` is the number of hyperedges of the hypergraph.
    pub fn get_weights_order(&self, order: usize) -> Option<Vec<f64>> {
        let mut res = Vec::new();
        // O(m)
        self.edge_list.values().for_each(|hyperedge| {
            if (*hyperedge.nodes).borrow().len() == order {
                res.push(hyperedge.weight);
            }
        });
        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    /// Returns the orders of all hyperedges in the hypergraph.
    ///
    /// The returned list may contains dupicates.
    ///
    /// # Returns
    /// - `Option<Vec<usize>>` - `Some` list with the orders of all hyperedges if there are hyperedges; `None` if  
    /// the hypergraph is empty.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` denotes the number of hyperedges.
    pub fn get_orders(&self) -> Option<Vec<usize>> {
        if self.edge_list.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            // O(m)
            self.edge_list.values().for_each(|hyperedge| {
                res.push((*hyperedge.nodes).borrow().len());
            });
            Some(res)
        }
    }

    /// Returns the max order of the hyperedges
    ///
    /// # Returns
    /// - `usize` - The max order.
    ///
    /// # Complexity
    /// - `O(m)`.
    pub fn max_order(&self) -> usize {
        match self.get_orders() {
            Some(val) => *val.iter().max().unwrap(),
            _ => 0,
        }
    }

    /// `type Node = i64`  
    /// `type IterNodes<'a> = std::collections::hash_map::Keys<'a, Node, AHashSet<EdgeID>>`   
    ///
    /// Gives an iterator through all the nodes of the hypergraph.
    ///
    /// # Returns
    /// - `IterNodes` - The iterator.
    ///
    /// # Complexity
    /// - `O(1)`.
    pub fn get_nodes(&self) -> IterNodes {
        self.incidence_list.keys()
    }

    /// `type Node = i64`  
    ///
    /// Gives the neighbors of a specific node.
    ///
    /// # Parameters
    /// - `node` : `Node` - The node.
    ///
    /// # Returns
    /// - `Option<Vec<Node>>` - `Some(nodes)` containing the list of neighbors of `node`. Returns `None` if the node   
    /// provided is not in the hypergraph.  
    ///
    /// # Complexity  
    /// - `O(n*m)`, where `n` and `m` are the number nodes and hyperedges, respectively, of the hypergraph.
    pub fn get_neighbors(&self, node: Node) -> Option<Vec<Node>> {
        match self.incidence_list.get(&node) {
            Some(incidence_list) => {
                let mut res = AHashSet::new();

                // O(m)
                for edge_id in incidence_list.iter() {
                    let edge_now = (*self.edge_list.get(edge_id).unwrap().nodes).borrow();
                    // O(n)
                    edge_now.iter().for_each(|v| {
                        res.insert(*v);
                    });
                }

                // We don't consider node itseld as a neighbor
                res.remove(&node);

                //O(n), but is necessary to not return a AHashSet
                Some(res.into_iter().collect::<Vec<Node>>())
            }
            _ => None,
        }
    }

    /// `type Node = i64`  
    ///
    /// Gives the neighbors of a specific node, considering only hyperdges with a specific order.
    ///
    /// # Parameters
    /// - `node` : `Node` - The node.
    /// - `order` : `usize` - The order of the hyperedges to consider.
    ///
    /// # Returns
    /// - `Option<Vec<Node>>` - `Some(nodes)` containing the list of neighbors of `node`, which are incident with at   
    /// least one hyperedge with the provided order. Returns `None` if `node` is not in the hypergraph.  
    ///
    /// # Complexity  
    /// - `O(n*m)`, where `n` and `m` are the number nodes and hyperedges, respectively, of the hypergraph.
    pub fn get_neighbors_with_order(&self, node: Node, order: usize) -> Option<Vec<Node>> {
        match self.incidence_list.get(&node) {
            Some(incidence_list) => {
                let mut res = AHashSet::new();

                // O(m)
                for edge_id in incidence_list.iter() {
                    let edge_now = (*self.edge_list.get(edge_id).unwrap().nodes).borrow();
                    if order == edge_now.len() {
                        // O(n)
                        edge_now.iter().for_each(|v| {
                            res.insert(*v);
                        });
                    }
                }

                res.remove(&node);

                //O(n), but is necessary to not return a AHashSet
                Some(res.into_iter().collect::<Vec<Node>>())
            }
            _ => None,
        }
    }

    /// `type Node = i64`  
    ///
    /// Get the hyperedges which are incident to a specific node.  
    ///
    /// # Parameters
    /// - `node` : `Node` - Node in the hypergraph.
    ///
    /// # Returns
    /// - `Option<Vec<Ref<Vec<Node>>>>` : `Some(edges)` containing immutable references to the hyperedges which are incident  
    /// to the given `node`. Returns `None` if the node does not exist in the hypergraph.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` is the number of hyperedges of the hyperegraph..
    pub fn get_incident_edges(&self, node: Node) -> Option<Vec<Ref<Vec<Node>>>> {
        match self.incidence_list.get(&node) {
            Some(incidence_list) => {
                let mut res = Vec::new();

                // O(m)
                incidence_list.iter().for_each(|edge_id| {
                    let hyperedge = self.edge_list.get(edge_id).unwrap();

                    // O(1)
                    res.push((*hyperedge.nodes).borrow())
                });
                Some(res)
            }
            _ => None,
        }
    }

    /// `type Node = i64`  
    ///
    /// Get the hyperedges with a specific order and which are incident to a specific node.  
    ///
    /// # Parameters
    /// - `node` : `Node` - Node in the hypergraph.
    /// - `order` : `usize` - The order of the edges of interest.
    ///
    /// # Returns
    /// - `Option<Vec<Ref<Vec<Node>>>>` : `Some(edges)` containing immutable references to the hyperedges that have the  
    /// specified `order` and are incident to the given `node`. Returns `None` if the node does not exist in the hypergraph.
    ///
    /// # Complexity
    /// - `O(m)`, where `m` is the number of hyperedges of the hyperegraph..
    pub fn get_incident_edges_with_order(
        &self,
        node: Node,
        order: usize,
    ) -> Option<Vec<Ref<Vec<Node>>>> {
        match self.incidence_list.get(&node) {
            Some(incidence_list) => {
                let mut res = Vec::new();

                // O(m)
                incidence_list.iter().for_each(|edge_id| {
                    if (*self.edge_list.get(edge_id).unwrap().nodes).borrow().len() == order {
                        let hyperedge = self.edge_list.get(edge_id).unwrap();

                        // O(1)
                        res.push((*hyperedge.nodes).borrow())
                    }
                });
                Some(res)
            }
            _ => None,
        }
    }

    /// `type Node = i64`  
    ///
    /// Add a node to the Hypergraph.
    ///
    /// # Parameters
    /// - `node`: Node
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn add_node(&mut self, node: Node) {
        if !self.incidence_list.contains_key(&node) {
            self.incidence_list.insert(node, AHashSet::new());
        }
    }

    /// `type Node = i64`  
    ///
    /// Add a list of nodes to the Hypergraph.
    ///
    /// # Parameters
    /// - `nodes`: `&[Node]` - List of nodes.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n)`, where `n` is the number of nodes provided.
    pub fn add_nodes(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.add_node(*node);
        }
    }

    /// Checks wheter the hypergraph is weighted.  
    ///
    /// # Returns
    /// - `bool` - `true` if the hypergraph is weighted, `false` otherwise.
    ///
    /// # Complexity  
    /// - `O(1)`
    pub fn is_weighted(&self) -> bool {
        self.weighted
    }

    /// `type Node = i64`  
    ///
    /// Check if an edge is in the hypergraph.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Edge to be checked.  
    ///
    /// # Returns
    /// - `bool` : `true` if `edge` is in the hypergraph, `false` otherwise.
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn check_edge(&self, edge: &Vec<Node>) -> bool {
        let edge_id = Self::compute_edge_id(edge);
        self.edge_list.contains_key(&edge_id)
    }

    /// Check if a node is in the hypergraph.
    ///
    /// # Parameters
    /// - `node` : `Node` - The node to be checked.  
    ///
    /// # Returns
    /// - `bool` : `true` if the node is in the hypergraph, `false` otherwise.  
    ///
    /// # Complexity
    /// - `O(1)`
    pub fn check_node(&self, node: Node) -> bool {
        self.incidence_list.contains_key(&node)
    }

    /// `type Node = i64`
    /// Add a hyperedge, with default weight set to 0, to the hypergraph.
    ///
    /// If the hyperedge was already present, then its weight is updated.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge to insert.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n)`, where `n` is the length of the hyperedge.
    pub fn add_edge(&mut self, edge: &Vec<Node>) {
        self.compute_add_edge(&edge.to_vec(), 0_f64);
    }

    /// `type Node = i64`
    ///
    /// Add a hyperedge to the hypergraph. If the hyperedge is already in the hypergraph, its weight is updated.  
    ///
    /// If the hyperedge was already present, then its weight is updated.  
    ///
    /// If the hypergraph is not weighted and a `weight > 0` is provided, then `weight` will be set to 0.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge to insert.
    /// - `weight` : `f64` - Weight of the hyperedge.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n)`, where `n` is the length of the hyperedge.
    pub fn add_edge_weighted(&mut self, edge: &Vec<Node>, mut weight: f64) {
        if !self.weighted {
            weight = 0_f64;
        }
        self.compute_add_edge(&edge.to_vec(), weight);
    }

    /// `type Node = i64`
    ///
    /// Add a list of hyperedges, with default weight set to 0, to the hypergraph.  
    ///
    /// If `edges` contains duplicates, the considered hyperedge, with its weight, will be the last encountered in the list. This
    /// does not affect the result, since every hyperedge in the list will have 0 as its weight.      
    ///
    /// If a hyperedge was already present, then its weight is updated.
    ///
    /// # Parameters
    /// - `edges` : `&[Vec<Node>]` - Hyperedges to insert.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(l*n)`, where `l` is the length of `edges`, `n` is the number of nodes.
    pub fn add_edges(&mut self, edges: &[Vec<Node>]) {
        for edge in edges.iter() {
            self.compute_add_edge(edge, 0_f64);
        }
    }

    /// `type Node = i64`
    ///
    /// Add a list of hyperedges to the hypergraph. If a hyperedge is already in the hypergraph, its weight is updated.
    ///
    /// Let `n`, `m` be the length of `edges` and `weights` respectively. Consider this three cases:   
    /// - `n` > `m`: then the first m hyperedges will receive its corresponding weight, and the last `n-m` will be   
    /// initialized with 0 as their weight;
    /// - `n` = `m`: then every hyperedge will receive its corresponding weight;
    /// - `n` < `m`: same as above; there will simply be some weights which will not be assigned.  
    ///
    /// If `_edge_list` contains duplicates, the considered hyperedge, with its weight, will be the first encountered in the list.   
    ///
    /// If a hyperedge was already present, then its weight is updated.
    ///
    /// # Parameters
    /// - `edges` : `&[Vec<Node>]` - Hyperedges to insert.
    /// - `weights` : `&[f64]` - Weights of the hyperedges.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n*m)`, where `n` is the max length of an edge, `m` is the number of hyperedges.
    pub fn add_edges_weighted(&mut self, edges: &[Vec<Node>], weights: &[f64]) {
        let mut index = 0;
        let mut next;

        for edge in edges.iter() {
            if index < weights.len() {
                next = weights[index];
            } else {
                next = 0_f64;
            }

            self.compute_add_edge(edge, next);
            index += 1;
        }
    }

    /// `type Node = i64`    
    ///
    /// Weakly deletion of a hyperedge from the hypergraph.    
    /// Weakly delete hyperedge 'e' from hypergraph `H = (V,E)` consists of removing `e` from `E`.  
    ///
    /// If the node provided is not in the hypergraph, nothing happens for it.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - The hyperedge to remove.
    ///
    /// # Returns
    /// - `bool` - `true` if the hyperedge was in the hypergraph, `false` otherwise.
    ///
    /// # Complexity
    /// - `O(n)`, where `n` is the order of the hyperedge provided, ie its length.
    pub fn remove_edge(&mut self, edge: &Vec<Node>) -> bool {
        let edge_id = Self::compute_edge_id(edge);

        if !self.edge_list.contains_key(&edge_id) {
            false
        } else {
            // Update incidence_list, O(n)
            for (_, edge_list) in self.incidence_list.iter_mut() {
                edge_list.remove(&edge_id);
            }

            // Update edge_list, O(1)
            self.edge_list.remove(&edge_id);

            true
        }
    }

    /// `type Node = i64`   
    ///
    /// Weakly deletion of a list of hyperedges from the hypergraph.  
    /// See `Self::remove_edge` for more details.   
    ///
    /// If the list provided contains hyperedges which are not in the hypergraph, nothing happens for them.
    ///
    /// # Parameters
    /// - `edges` : `&[Vec<Node>]` - List of hyperedges to remove.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n*l)`, where `n` is the number of nodes, `l` is the length of `edges`. We are assuming that the list provided  
    /// contains only hyperedges which are in the hypergraph.
    pub fn remove_edges(&mut self, edges: &[Vec<Node>]) {
        // O(m)
        for edge in edges.iter() {
            self.remove_edge(edge);
        }
    }
    /// `type Node = i64`.    
    ///
    /// Weakly removes a node from the hypergraph.  
    /// Weakly deletion of node `v` from hypergraph `H = (V,E)` consists of removing `v` from `V` and from every hyperedge   
    /// `E` such that `v` is in `E`.  
    ///
    /// If the node provided is not in the hypergraph, nothing happens for it.  
    ///
    /// # Parameters
    /// - `node` : `Node` - Node to be removed.
    ///
    /// # Returns
    /// - `bool` - `true` if the node was in the hypergraph, `false` otherwise.
    ///
    /// # Complexity
    /// - `O(n*m)`, where `n` is the number of nodes, `m` is the number of hyperedges.
    ///
    /// # Notes   
    /// If we would have used a hash-based collection, we could achieve this in `O(m)`.
    pub fn remove_node(&mut self, node: Node) -> bool {
        if !self.incidence_list.contains_key(&node) {
            false
        } else {
            // Update incidence_list, O(1)
            let edges = self.incidence_list.remove(&node).unwrap();

            // Update edge_list, O(n*m)
            for edge_id in edges.iter() {
                // O(m)
                // O(n)
                let mut index_node = 0;
                // Search the node provided, which is removed. O(n). --> If we would use a hash collection, this would be O(1)
                let mut edge_now = (*self.edge_list.get(edge_id).unwrap().nodes).borrow_mut();

                for index in 0..edge_now.len() {
                    if edge_now[index] == node {
                        index_node = index;
                        break;
                    }
                }

                // Removes `node` from the hyperedge, O(n)
                edge_now.remove(index_node);
            }

            true
        }
    }

    /// `type Node = i64`  
    ///
    /// Weakly removes a list of nodes from the hypergraph. See `Self::remove_node` for more details.   
    ///
    /// If the list provided contains nodes which are not in the hypergraph, nothing happens for them.
    ///
    /// # Parameters
    /// - `nodes` : `&[Node]` - List of the nodes to be removed.
    ///
    /// # Returns
    /// - `()`
    ///
    /// # Complexity
    /// - `O(l*n*m)`, where `l` is the length of the list of nodes, `n` is the number of nodes, `m` is the   
    /// number of edges. We are assuming that the list provided contains only nodes which are in the hypergraph.  
    pub fn remove_nodes(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.remove_node(*node);
        }
    }

    /// `type Node = i64`  
    ///
    /// Strongly remove a node from the hypergraph.   
    ///
    /// If the node provided is not in the hypergraph, nothing happens for it.  
    ///
    /// # Parameters
    /// - `node` : `Node` - Node to be removed.
    ///
    /// # Returns
    /// - `bool` : `true` if the node was in the hypergraph, `false` otherwise.
    ///
    /// # Complexity
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges in the hypergraph, respectively.
    pub fn strong_remove_node(&mut self, node: Node) -> bool {
        if !self.incidence_list.contains_key(&node) {
            false
        } else {
            // Update incidence_list, O(1)
            let edges = self.incidence_list.remove(&node).unwrap();

            // We could have re-used the function Self::remove_edges, but this is more efficient, because it does not require to
            // convert `edges`, which is AHashSet<EdgeID> to a &Vec<&Vec<Node>>, which is O(m).

            // Update incidence_list, O(n*m)
            for (_, set) in self.incidence_list.iter_mut() {
                for edge_id in edges.iter() {
                    set.remove(edge_id);
                }
            }

            // Update edge_list, O(m)
            for edge_id in edges.iter() {
                self.edge_list.remove(edge_id);
            }

            true
        }
    }
    /// `type Node = i64`    
    ///
    /// Strongly removes a list of nodes from the hypergraph.  
    /// See `Hypergraph::strong_remove_node` for more details.  
    ///
    /// If the list provided contains nodes which are not in the hypergraph, nothing happens for them.
    ///
    /// # Parameters
    /// - `nodes` : `&[Node]` - List of the nodes to be removed.
    ///
    /// # Returns  
    /// - `()`  
    ///
    /// # Complexity
    /// - `O(l*n*m)`, where `l` is the length of the list `nodes`, `n` is the number of nodes, `m` is the   
    /// number of edges. We are assuming that the list provided contains only nodes which are in the hypergraph.
    pub fn strong_remove_nodes(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.strong_remove_node(*node);
        }
    }

    /// `type IterEdges<'a> = std::collections::hash_map::Values<'a, u64, Hyperedge>`   
    ///
    /// Gives an iterator over the hyperedges in the hypergraph.   
    ///
    /// The hyperedges come also with their weight.
    ///
    /// # Returns
    /// - `IterEdges` : The iterator over the hyperedges, which are stored as `Hyperedge`.
    ///
    /// # Complexity  
    /// - `O(1)`
    pub fn iter_edges(&self) -> IterEdges {
        // This iterator, as specified by the lifetime symbol '_', is an iterator over borrowed values, so
        // it does not take ownership
        self.edge_list.values().into_iter()
    }

    /// Checks wether the hypergraph is uniform, ie all hyperedges have the same order.
    ///
    /// By definition, a hypergraph with 0 hyperedges is 0-uniform.
    ///
    /// # Returns
    /// - `Option<usize>`: `Some(usize)` if it is uniform, with the "uniform value" stored in, `None` otherwise.
    ///
    /// # Complexity  
    /// - `O(m)`, where `m` is the number of hyperedges.
    pub fn is_uniform(&self) -> Option<usize> {
        if self.edge_list.len() == 0 {
            Some(0)
        } else {
            let mut edges = self.edge_list.values().into_iter();
            // Order of the "first" hyperedge in edge_list
            let length = (*edges.next().unwrap().nodes).borrow().len();

            for edge in edges {
                if (*edge.nodes).borrow().len() != length {
                    return None;
                }
            }
            Some(length)
        }
    }

    /// Clears the Hypergraph, removing all the nodes and the hyperedges.   
    ///
    /// Keeps the allocated memory for reuse. Also, if the hypergraph was weighted, it remains weighted.
    ///
    /// # Returns
    /// - `()`
    pub fn clear(&mut self) {
        self.incidence_list.clear();
        self.edge_list.clear();
    }

    /*
    ===============================================================================
    |                       PRIVATE HELPER FUNCTIONS                              |
    ===============================================================================
    */

    /// `type Node = i64`
    ///
    /// Effectively computes the (weigted) add of a hyperedge to the hypergraph.
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge to be inserted.
    /// - `weight` : `f64` - Weight of the hyperedge.
    ///
    /// # Returns  
    /// - `()`
    ///
    /// # Complexity
    /// - `O(n)`, where `n` is the number of nodes.
    fn compute_add_edge(&mut self, edge: &Vec<Node>, weight: f64) {
        let edge_id = Self::compute_edge_id(edge);

        if !self.edge_list.contains_key(&edge_id) {
            // Edge not already in

            // Update edge_list, O(1)
            let hyperedge = Hyperedge::new(Rc::new(RefCell::new(edge.clone())), weight);
            self.edge_list.insert(edge_id, hyperedge);

            // Update incidence_list, O(n)
            for node in edge.iter() {
                self.incidence_list
                    .entry(*node)
                    .and_modify(|set| {
                        set.insert(edge_id);
                    })
                    .or_insert_with(|| {
                        let mut set = AHashSet::new();
                        set.insert(edge_id);
                        set
                    });
            }
        } else {
            // If the edge is already in, its weight is updated
            self.edge_list.entry(edge_id).and_modify(|hyperedge| {
                hyperedge.set_weight(weight);
            });
        }
    }

    /// Creates the edgeID for a Hyperedge.  
    /// `type EdgeID = u64`    
    /// `type Node = i64`
    ///
    /// # Parameters  
    /// - `edge` : `Vec<Node>` - hyperedge for which the edgeID is needed.
    ///
    /// # Returns
    /// - `u64`- The computed edgeID  
    ///
    /// # Complexity  
    /// - The implementation of the hashing function for `Vec<T>` is the one of the standard library, so `O(n)`, where `n` is the   
    /// length of the array. (?)
    fn compute_edge_id(edge: &Vec<Node>) -> EdgeID {
        let hasher_factory = RandomState::with_seeds(SEED1, SEED2, SEED3, SEED4);
        let mut hasher = hasher_factory.build_hasher();
        edge.hash(&mut hasher);

        hasher.finish()
    }
}


/*
    TODO
    1. Finish the doc (Hypergraph struct, Hyperedge struct, etc.);
    2. implement other functions from `hypergraphx`;
    3. implement the tests;
    4. see for improvments.
*/