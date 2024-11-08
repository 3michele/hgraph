mod hyperedge;
mod hypergraph_traits;
pub mod visits;
mod cc;

// One of the fastest and secure non cryptographic hash for rust
use ahash::{AHashMap, AHashSet, RandomState};

use std::hash::{BuildHasher, Hash, Hasher};

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

/// Core struct to represent a hypergraph.   
/// Hypergraphs are a generalization of graphs, where each edge can connect multiple nodes
/// (see [Hypergraph](https://en.wikipedia.org/wiki/Hypergraph)).
///
/// # Design Overview
/// This implementation optimizes for **memory efficiency** and **performance** using a **double-hashing** approach.
///
/// #### Hyperedge Identification
///   Each hyperedge is represented as a set of nodes and is assigned a unique `EdgeID`, computed through an initial hash.  
///   This unique identifier allows for `O(1)` accesses, and solves the performance overhead associated with repeatedly  
///   hashing entire node collections, which would be `O(n)` on the length `n` of the collection.
///     
/// #### Efficient Storage  
///   The `edge_list` hashmap stores hyperedges by mapping each `EdgeID` to its corresponding `Hyperedge`. This design   
///   reduces memory usage by only storing identifiers in `incidence_list`, allowing nodes to reference hyperedges without  
///   duplicating data. Thus, the hypergraph can efficiently handle large collections of nodes and edges without excessive   
///   memory consumption.
///
/// # User Interaction
/// The user communicates via hyperedges, not `EdgeID`'s, meaning that he will provide a concrete set of nodes whenever he  
/// calls a method which requires a hyperedge. Internally, the hypergraph computes the `EdgeID` for the hyperedge provided,  
/// and operates on that ID.
pub struct Hypergraph {
    /// States if the hypergraphs is weighted.
    weighted: bool,

    /// Maps each node to a set of `EdgeID`s of the hyperedges it connects to.
    /// This efficient storage mechanism reduces memory usage by avoiding the need
    /// to store full sets of edges for each node, enabling faster operations.
    incidence_list: AHashMap<Node, AHashSet<EdgeID>>,

    /// Maps each `EdgeID` to its associated `Hyperedge`.
    /// By storing hyperedges indexed by their unique IDs, this design allows for
    /// rapid access to hyperedge data without redundant storage, with a concrete `O(1)` hash.
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

    /// `type Node = i64`
    ///
    /// Creates an unweighted `Hypergraph` from a list of hyperedges.  
    ///
    /// For every duplicate in `_edge_list` there will be only an hyperedge.  
    ///
    /// # Parameters
    /// - `_edge_list`: (`&[Vec<Node>]`) - List of hyperedges, each represented as a vector of nodes.
    ///
    /// # Returns
    /// - `Self` - A new instance of `Hypergraph`.
    pub fn from(_edge_list: &[Vec<Node>]) -> Self {
        let mut result = Self::new(false);

        for edge in _edge_list.iter() {
            let edge_id = Self::compute_edge_id(edge);

            if !result.edge_list.contains_key(&edge_id) {
                Self::compute_add_edge(&mut result, edge, 0_f64);
            }
        }
        result
    }

    /// `type Node = i64`
    ///
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
    pub fn from_weighted(_edge_list: &[Vec<Node>], weights: &[f64]) -> Self {
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
    /// # Performance
    /// - `O(1)`
    pub fn num_nodes(&self) -> usize {
        self.incidence_list.len()
    }

    /// Returns the number of hyperedges in the hypergraph.
    ///
    /// # Returns
    /// - `usize` - Size of the hypergraph.
    ///
    /// # Performance
    /// - `O(1)`
    pub fn num_edges(&self) -> usize {
        self.edge_list.len()
    }
    
    /// Returns the number of hyperedges with an order/size less then or equal to the one provided.  
    /// 
    /// The convention is `order == size - 1`.
    ///
    /// # Parameters
    /// - `order` : `Option<usize>` - Order of interest, optional.
    /// - `size` : `Option<usize>` - Size of interest, optional.
    /// - `up_to` : `bool` - If `true`, then the hyperedges considered are the ones which respect the `≤` relation, with respect   
    /// to their order/size. Otherwise the choice is based on the `=` relation.
    ///
    /// # Returns
    /// - `Result<usize, &str>` - `Ok` containing the number of selected hyperedges, if one, and only one, between `order`   
    /// and `size` is provided. `Err` containing an error message otherwise. 
    ///
    /// # Performance
    /// - `O(m)`, where `m` denotes the number of hyperedges of the hypergraph.
    pub fn num_edges_with(&self, order: Option<usize>, size: Option<usize>, up_to: bool) -> Result<usize, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified") 
        } else if order == None && size == None {
            Err("At least one between orders and sizes should be specified")
        } else {
            let mut res = 0;

            let filter = if let Some(val) = order {
                val + 1
            } else {
                size.unwrap()
            };

            if up_to {
                for (_, edge) in self.edge_list.iter() {
                    if edge.nodes.len() <= filter {
                        res += 1;
                    }
                }
            } else {
                for (_, edge) in self.edge_list.iter() {
                    if edge.nodes.len() == filter {
                        res += 1;
                    }
                }
            }

            Ok(res)
        }
    }

    /// Returns the weight of a specific hyperedge.
    ///
    /// # Parameters
    /// - 'edge' : `&Vec<Node>` - The Hyperedge.
    ///
    /// # Returns
    /// - `Option<f64>` - `Some` weight of the hyperedge. Returns `None` if the hyperedge is not in the hypergraph.
    ///
    /// # Performance
    /// - `O(1)`
    pub fn get_weight(&self, edge: &Vec<Node>) -> Option<f64> {
        let edge_id = Self::compute_edge_id(edge);

        match self.edge_list.get(&edge_id) {
            Some(edge) => Some((*edge).weight),
            _ => None,
        }
    }

    /// `type Node = i64`
    ///
    /// Sets the weight of a specific hyperedge.
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge for which the weight has to be modified.
    /// - `new_weight` : `f64` - The new weight for the hyperedge.
    ///
    /// # Returns
    /// - `Result<f64, ()>` : `Ok` containing the previous weight of the provided hyperedge, if it exists in the hypergraphs.   
    /// Returns `Err` containing `()` if the specified hyperedge is not in the hypergraph.
    ///
    /// # Performance
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

    /// Returns the weights of all hyperedges.  
    ///
    /// The returned list may contain duplicates of the weights.
    ///
    /// # Returns
    /// - `Option<Vec<f64>>` - `Some` list with the weights if there are hyperedges. Returns `None` if the hypergraph has no hyperedges.
    ///
    /// # Performance
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

    /// Returns the weights of the selected hyperedges.
    /// 
    /// The convention is `order == size - 1`  
    ///
    /// The returned list may contain duplicates of the weights.
    ///
    /// # Parameters
    /// - `order` : `Option<usize>` - The order of interest (optional).
    /// - `size` : `Option<usize>` - The size of interest (optional).
    /// - `up_to` : `bool` - If `true`, it specifies to consider hyperedges with order/size less than or equal to the provided   
    /// order\size. If `false` the method considers only hyperedges with an equal order/size to the order/size provided.
    ///
    /// # Returns
    /// - `Result<Option<Vec<f64>>, &str>` - `Ok` containing `Some` list with the weights of the selected hyperedges, or    
    /// containing `None` if no such hyperedges exist, if one, and only one, between `order` and `size` is provided.   
    /// Returns `Err` containing an error message otherwise.
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges of the hypergraph.
    pub fn get_weights_with(&self, order: Option<usize>, size: Option<usize>, up_to: bool) -> Result<Option<Vec<f64>>, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified")
        } else if order == None && size == None {
            Err("Order and size cannot be both None")
        } else {
            let mut res = Vec::new();

            let filter = if let Some(val) = order {
                val + 1
            } else {
                size.unwrap()
            };

            // O(m)
            if up_to {
                self.edge_list.values().for_each(|hyperedge| {
                    if hyperedge.nodes.len() <= filter {
                        res.push(hyperedge.weight);
                    }
                });
            } else {
                self.edge_list.values().for_each(|hyperedge| {
                    if hyperedge.nodes.len() == filter {
                        res.push(hyperedge.weight);
                    }
                })
            }
    
            if res.is_empty() {
                Ok(None)
            } else {
                Ok(Some(res))
            }
        }
    }

    /// Returns the orders of all hyperedges in the hypergraph.
    ///
    /// The returned list may contains dupicates.
    ///
    /// # Returns
    /// - `Option<Vec<i64>>` - `Some` list with the orders of all hyperedges if there are hyperedges; `None` if  
    /// the hypergraph is empty.
    ///
    /// # Performance
    /// - `O(m)`, where `m` denotes the number of hyperedges.
    pub fn get_orders(&self) -> Option<Vec<i64>> {
        if self.edge_list.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            // O(m)
            self.edge_list.values().for_each(|hyperedge| {
                res.push(hyperedge.nodes.len() as i64 -1);
            });
            Some(res)
        }
    }

    /// Returns the sizes of all hyperedges in the hypergraph.
    ///
    /// The returned list may contains dupicates.
    ///
    /// # Returns
    /// - `Option<Vec<usize>>` - `Some` list with the orders of all hyperedges if there are hyperedges; `None` if  
    /// the hypergraph is empty.
    ///
    /// # Performance
    /// - `O(m)`, where `m` denotes the number of hyperedges.
    pub fn get_sizes(&self) -> Option<Vec<usize>> { 
        if self.edge_list.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            // O(m)
            self.edge_list.values().for_each(|hyperedge| {
                res.push(hyperedge.nodes.len());
            });
            Some(res)
        }
    }

    /// Returns the maximum order of the hyperedges.  
    /// 
    /// By convention, if the hyperedges has `max_size == 0`, then `max_order := 0`.
    ///
    /// # Returns
    /// - `usize` - The max order.
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges in the hypergraph.
    pub fn max_order(&self) -> usize {
        // It can be < 0 (-1)
        self.max_size().saturating_sub(1)
    }

    /// Returns the maximum order of the hyperedges.
    ///
    /// # Returns
    /// - `usize` - The max order.
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges in the hypergraph.
    pub fn max_size(&self) -> usize {
        match self.get_sizes() {
            Some(val) => *val.iter().max().unwrap(),
            _ => 0,
        }
    }

    /// `type Node = i64`  
    ///
    /// Returns a list with all the nodes of the hypergraph.
    ///
    /// # Returns
    /// - `Option<Vec<Node>>` - The list containing all the nodes of the hyperegraph.
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the number of nodes of the hypergraph.
    pub fn get_nodes(&self) -> Vec<Node> {
        let mut res = Vec::new();
        self.incidence_list.keys().for_each(|node_id| {
            res.push(*node_id);
        });

        res 
    }

    /// `type Node = i64`  
    /// 
    /// Returns the list of all hyperedges in the hypergraph.   
    /// 
    /// # Returns 
    /// - `Option<Vec<&Vec<Node>>>` - `Some` list of references to all the hyperedges if at least one of them exists in   
    /// the hypergraph. `None` otherwise. 
    /// 
    /// # Performance
    /// - `O(m)`
    pub fn get_edges(&self) -> Option<Vec<&Vec<Node>>> {
        if self.edge_list.is_empty() {
            None 
        } else {
            let mut res = Vec::new();

            self.edge_list.values().for_each(|hyperedge| {
                res.push(&hyperedge.nodes);
            });
            Some(res)
        }
    }

    /// Returns references of the selected hyperedges.
    /// 
    /// The convention is `order == size - 1`  
    ///
    /// # Parameters
    /// - `order` : `Option<usize>` - The order of interest (optional).
    /// - `size` : `Option<usize>` - The size of interest (optional).
    /// - `up_to` : `bool` - If `true`, it specifies to consider hyperedges with order/size less than or equal to the provided   
    /// order\size. If `false` the method considers only hyperedges with an equal order/size to the order/size provided.
    ///
    /// # Returns
    /// - `Result<Option<Vec<&Vec<Node>>>, &str>` - `Ok` containing `Some` list with the references of the selected hyperedges, or    
    /// containing `None` if no such hyperedges exist, if one, and only one, between `order` and `size` is provided.   
    /// Returns `Err` containing an error message otherwise.
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges of the hypergraph.
    pub fn get_edges_with(&self, order: Option<usize>, size: Option<usize>, up_to: bool) -> Result<Option<Vec<&Vec<Node>>>, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified")
        } else if order == None && size == None {
            Err("Order and size cannot be both None")
        } else {
            let mut res = Vec::new();

            let filter = if let Some(val) = order {
                val + 1
            } else {
                size.unwrap()
            };

            self.edge_list.values().for_each(|hyperedge| {
                if up_to && hyperedge.nodes.len() <= filter {
                    res.push(&hyperedge.nodes);
                } else if !up_to && hyperedge.nodes.len() == filter {
                    res.push(&hyperedge.nodes)
                }
            });
    
            if res.is_empty() {
                Ok(None)
            } else {
                Ok(Some(res))
            }
        }
    }

    /// `type Node = i64`  
    /// 
    /// Gives the neighbors of a specific node.  
    /// 
    /// The convention is `order == size - 1`. 
    ///
    /// # Parameters
    /// - `node` : `Node` - The node of interest.
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. 
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. 
    ///
    /// # Returns
    /// - `Result<Option<Vec<Node>>, &str>` - `Ok` containing `Some` list of neighbors of `node`, or containing `None` if   
    /// the node provided is not in the hypergraph. Returns `Err` containing an error message if both `order` and `size`    
    /// are provided.
    ///
    /// # Performance  
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and hyperedges, respectively, of the hypergraph.
    pub fn get_neighbors(&self, node: Node, order: Option<usize>, size: Option<usize>) -> Result<Option<Vec<Node>>, &str> {
        // Both order and size are specified
        if order != None && size != None {
            Err("Order and size cannot be both specified")
        } else {
            match self.incidence_list.get(&node) {
                    Some(incidence_list) => {
                        let mut res = AHashSet::new();

                        // Both order and size are not specified
                        if order == None && size == None {
                            for edge_id in incidence_list.iter() {
                                let edge_now = &self.edge_list.get(edge_id).unwrap().nodes;
                                edge_now.iter().for_each(|v| {
                                    res.insert(*v);
                                });
                            }
                        } else {
                            let filter = if let Some(val) = order {
                                // Only Order is specified
                                val + 1
                            } else {
                                // Only Size is specified
                                size.unwrap()
                            };

                            for edge_id in incidence_list.iter() {
                                let edge_now = &self.edge_list.get(edge_id).unwrap().nodes;
                                if edge_now.len() == filter {
                                    edge_now.iter().for_each(|v| {
                                        res.insert(*v);
                                    });
                                }
                            }
                        }
                        // We don't consider the node itself as a neighbor
                        res.remove(&node);

                        //O(n), but is necessary to not return a AHashSet
                        Ok(Some(res.into_iter().collect::<Vec<Node>>()))
                    },
                 _ => Ok(None),
            }
        }
    }

    /// `type Node = i64`  
    ///
    /// Get the hyperedges which are incident to a specific node.    
    /// 
    /// The convention is `order == size - 1`. 
    ///
    /// # Parameters
    /// - `node` : `Node` - Node in the hypergraph.
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. 
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. 
    ///
    /// # Returns
    /// - `Result<Option<Vec<&Vec<Node>>>, &str>` : `Ok` containing `Some` immutable references to the hyperedges which are   
    /// incident to the given `node`, or containing `None` if the node does not exists in the hypergraph. Returns `Err` containing  
    /// an error message if both `order` and `size` are provided. 
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges of the hyperegraph.
    pub fn get_incident_edges(&self, node: Node, order: Option<usize>, size: Option<usize>) -> Result<Option<Vec<&Vec<Node>>>, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified")    
        } else {
            match self.incidence_list.get(&node) {
                Some(incidence_list) => {
                    let mut res = Vec::new();

                    // Both order and size are not specified
                    if order == None && size == None {
                        // O(m)
                        incidence_list.iter().for_each(|edge_id| {
                            let hyperedge = self.edge_list.get(edge_id).unwrap();

                            res.push(&hyperedge.nodes)
                        });
                    } else {
                        let filter = if let Some(val) = order {
                            // Only order is specified 
                            val + 1
                        } else {
                            // Only Size is specified 
                            size.unwrap()
                        };
                        // O(m)
                        incidence_list.iter().for_each(|edge_id| {
                            if (&self.edge_list.get(edge_id).unwrap().nodes).len() == filter {
                                let hyperedge = self.edge_list.get(edge_id).unwrap();
        
                                res.push(&hyperedge.nodes)
                            }
                        });
                    }
                    
                    Ok(Some(res))
                }
                _ => Ok(None),
            }
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
    /// - `bool` - `true` if the node was not already in the hypergraph, `false` otherwise.
    ///
    /// # Performance
    /// - `O(1)`
    pub fn add_node(&mut self, node: Node) -> bool {
        if !self.incidence_list.contains_key(&node) {
            self.incidence_list.insert(node, AHashSet::new());
            true 
        } else {
            false 
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
    /// - `bool` - `true` if all the nodes were not already in the hypergraph, `false` otherwise. 
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the number of nodes provided.
    pub fn add_nodes(&mut self, nodes: &[Node]) -> bool {
        let mut res = true;

        for node in nodes.iter() {
            res &= self.add_node(*node);
        }
        res 
    }

    /// Checks wheter the hypergraph is weighted.  
    ///
    /// # Returns
    /// - `bool` - `true` if the hypergraph is weighted, `false` otherwise.
    ///
    /// # Performance  
    /// - `O(1)`
    pub fn is_weighted(&self) -> bool {
        self.weighted
    }

    /// `type Node = i64`  
    ///
    /// Check if a hyperedge is in the hypergraph.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge to be checked.  
    ///
    /// # Returns
    /// - `bool` : `true` if `edge` is in the hypergraph, `false` otherwise.
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the number of nodes of the hypergraph.
    pub fn check_edge(&self, edge: &Vec<Node>) -> bool {
        let edge_id = Self::compute_edge_id(edge); 
        self.edge_list.contains_key(&edge_id)
    }

    /// Check if a node is in the hypergraph.
    ///
    /// # Parameters
    /// - `node` : `Node` - The node to be checked.  
    /// # Returns
    /// - `bool` : `true` if the node is in the hypergraph, `false` otherwise.  
    ///
    /// # Performance
    /// - `O(1)`
    pub fn check_node(&self, node: Node) -> bool {
        self.incidence_list.contains_key(&node)
    }

    /// `type Node = i64`   
    /// 
    /// Add a hyperedge, with default weight set to 0, to the hypergraph.
    ///
    /// If the hyperedge was already present, then its weight is updated.  
    ///
    /// # Parameters
    /// - `edge` : `&Vec<Node>` - Hyperedge to insert.
    ///
    /// # Returns
    /// - `bool` - `false` if the hyperedge was already in, `true` otherwise. 
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the length of the hyperedge.
    pub fn add_edge(&mut self, edge: &Vec<Node>) -> bool {
        Self::compute_add_edge(self, &edge.to_vec(), 0_f64)
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
    /// - `bool` - `false` if the hyperedge was already in, `true` otherwise. 
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the length of the hyperedge.
    pub fn add_edge_weighted(&mut self, edge: &Vec<Node>, mut weight: f64) -> bool {
        if !self.weighted {
            weight = 0_f64;
        }
        Self::compute_add_edge(self,&edge.to_vec(), weight) 
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
    /// - `bool` - `true` if all hyperedges were not already in, `false` otherwise.
    ///
    /// # Performance
    /// - `O(l*n)`, where `l` is the length of `edges`, `n` is the number of nodes.
    pub fn add_edges(&mut self, edges: &[Vec<Node>]) -> bool {
        let mut res = true;
        for edge in edges.iter() {
            res &= Self::compute_add_edge(self, edge, 0_f64);
        }
        res 
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
    /// If `edges` contains duplicates, the considered hyperedge, with its weight, will be the last encountered in the list.   
    ///
    /// If a hyperedge was already present, then its weight is updated.
    ///
    /// # Parameters
    /// - `edges` : `&[Vec<Node>]` - Hyperedges to insert.
    /// - `weights` : `&[f64]` - Weights of the hyperedges.
    ///
    /// # Returns
    /// - `bool` - `true` if all hyperedges were not already in, `false` otherwise.
    ///
    /// # Performance
    /// - `O(n*m)`, where `n` is the max length of an edge, `m` is the number of hyperedges.
    pub fn add_edges_weighted(&mut self, edges: &[Vec<Node>], weights: &[f64]) -> bool {
        let mut index = 0;
        let mut next;
        let mut res = true;

        for edge in edges.iter() {
            if index < weights.len() {
                next = weights[index];
            } else {
                next = 0_f64;
            }

            res &= Self::compute_add_edge(self, edge, next);
            index += 1;
        }
        res 
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
    /// # Performance
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
    /// - `bool` - `true` if all the hyperedges provided were in the hypergraph, `false` otherwise. 
    ///
    /// # Performance
    /// - `O(n*l)`, where `n` is the number of nodes, `l` is the length of `edges`. We are assuming that the list provided  
    /// contains only hyperedges which are in the hypergraph.
    pub fn remove_edges(&mut self, edges: &[Vec<Node>]) -> bool {
        let mut res = true;

        // O(m)
        for edge in edges.iter() {
            res &= self.remove_edge(edge);
        }
        res 
    }

    // =======================================================================
    //                      We need to update the EdgeID'a
    // =======================================================================
    /// `type Node = i64`.    
    ///
    /// Weakly removes a node from the hypergraph.  
    ///
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
    /// # Performance
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

            // O(m)
            for edge_id in edges.iter() {
                // O(n)
                let mut edge_now = self.edge_list.get(edge_id).unwrap().clone();

                // O(n)
                self.remove_edge(&edge_now.nodes);

                // O(n)
                edge_now.nodes.retain(|x| *x != node);

                // O(n)
                self.add_edge_weighted(&edge_now.nodes, edge_now.weight);
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
    /// # Performance
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
    /// Strongly deletion of node `v` from hypergraph `H = (V,E)` constists of removing `v` from `V` and remove all `e` from `E`   
    /// sucht that `v` is in `e`.
    ///
    /// If the node provided is not in the hypergraph, nothing happens for it.  
    ///
    /// # Parameters
    /// - `node` : `Node` - Node to be removed.
    ///
    /// # Returns
    /// - `bool` : `true` if the node was in the hypergraph, `false` otherwise.
    ///
    /// # Performance
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
    ///
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
    /// # Performance
    /// - `O(l*n*m)`, where `l` is the length of the list `nodes`, `n` is the number of nodes, `m` is the   
    /// number of edges. We are assuming that the list provided contains only nodes which are in the hypergraph.
    pub fn strong_remove_nodes(&mut self, nodes: &[Node]) {
        for node in nodes.iter() {
            self.strong_remove_node(*node);
        }
    }

    /// `type Node = i64`   
    ///
    /// Returns a subhypergraph induced by the nodes in the list.   
    ///
    /// # Parameters
    /// - `nodes` : `&Vec<Node>` - List of nodes to be included in the subhypergraph.
    ///
    /// # Returns
    /// - `Self` - Induced subhypergraph.  
    ///
    /// # Performance
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the original hypergraph.
    pub fn subhypergraph(&self, nodes: &Vec<Node>) -> Self {
        let mut res = Self::new(self.weighted);

        // O(n)
        res.add_nodes(nodes);

        let nodes_as_set = Self::compute_vec_to_set(nodes);

        // O(m)
        for edge in self.edge_list.values() {
            // O(n)
            let edge_as_set = Self::compute_vec_to_set(&edge.nodes);

            // O(n)
            if edge_as_set.is_subset(&nodes_as_set) {
                res.add_edge_weighted(&edge.nodes, edge.weight);
            }
        }

        res
    }

    /// Returns a subhypergraph induced by the hyperedges of a specific order.
    ///
    /// # Parameters
    /// - `orders` : `Option<&Vec<usize>>` - List of orders of the hyperedges to be included in the subhypergraph (optional).
    /// - `sizes` : `Option<&Vec<usize>>` - List of sizes of the hyperedges to be included in the subhypergraph (optional).
    /// - `keep_nodes` : `bool` - If `true`, the nodes of the original hypergraph are kept in the subhypergraph. If `false`,  
    ///  only the hyperedges are kept. 
    ///
    /// # Returns
    /// - `Result<Self>` - `Ok` containing the induced subhypergraph if one, and exactly one, between `orders` and `sizes`   
    /// is provided. `Err` containing an error message otherwise. 
    ///
    /// # Performance
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and hyperedges, respectively, of the original hypergraph.
    pub fn subhypergraph_by_orders(&self, orders: Option<&Vec<usize>>, sizes: Option<&Vec<usize>>, keep_nodes: bool) -> Result<Self, &str> {
        if orders == None && sizes == None {
            Err("At least one between orders and sizes should be specified")
        } else if orders != None && sizes != None {
            Err("Orders and sizes cannot be both specified")
        } else {
            let mut res = Hypergraph::new(self.weighted);

            if keep_nodes {
                res.add_nodes(&self.get_nodes());
            }

            let mut filter_set = AHashSet::new();
            if let Some(val) = orders {
                for order in val.iter() {
                    filter_set.insert(*order + 1);
                }
            } else {
                for size in sizes.unwrap().iter() {
                    filter_set.insert(*size);
                }
            }
            

            // O(m)
            for hyperedge in self.edge_list.values() {
                // O(1)
                if filter_set.contains(&hyperedge.nodes.len()) {
                    // O(n)
                    res.add_edge_weighted(&hyperedge.nodes, hyperedge.weight);
                }
            }

            Ok(res)
        }
    }

    /// Returns the distribution of the orders of the hyperedges in the hypergraph.
    ///
    /// # Returns
    /// - `AHashMap<usize, usize>` - The dictionary which stores the orders as keys, and the number of occurrences for that   
    /// specific order as values.
    ///
    /// # Performance
    /// - `O(m)`, where `m` is the number of hyperedges in the hypergraph.
    pub fn distrbution_orders(&self) -> AHashMap<usize, usize> {
        let mut res = AHashMap::new();

        for hyperedge in self.edge_list.values() {
            res.entry(hyperedge.nodes.len())
                .and_modify(|total: &mut usize| {
                    *total += 1;
                })
                .or_insert(1);
        }
        res
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
    /// # Performance  
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
    /// # Performance  
    /// - `O(m)`, where `m` is the number of hyperedges.
    pub fn is_uniform(&self) -> Option<usize> {
        if self.edge_list.len() == 0 {
            Some(0)
        } else {
            let mut edges = self.edge_list.values().into_iter();
            // Order of the "first" hyperedge in edge_list
            let length = edges.next().unwrap().nodes.len();

            for edge in edges {
                if edge.nodes.len() != length {
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
    /// # Performance
    /// - `O(n)`, where `n` is the number of nodes.
    fn compute_add_edge(hg: &mut Hypergraph, edge: &Vec<Node>, weight: f64) -> bool {
        let edge_id = Self::compute_edge_id(edge);

        if !hg.edge_list.contains_key(&edge_id) {
            // Edge not already in

            // Update edge_list, O(1)
            let hyperedge = Hyperedge::new(edge.clone(), weight);
            hg.edge_list.insert(edge_id, hyperedge);

            // Update incidence_list, O(n)
            for node in edge.iter() {
                hg.incidence_list
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
            true 
        } else {
            // If the edge is already in, its weight is updated
            hg.edge_list.entry(edge_id).and_modify(|hyperedge| {
                hyperedge.set_weight(weight);
            });
            false  
        }
    }

    /// `type EdgeID = u64`    
    /// `type Node = i64`
    ///
    /// Effectively computes the edgeID for a Hyperedge.  
    ///
    /// # Parameters  
    /// - `edge` : `Vec<Node>` - hyperedge for which the edgeID is needed.
    ///
    /// # Returns
    /// - `u64`- The computed edgeID  
    ///
    /// # Performance  
    /// - The implementation of the hashing function for `Vec<T>` is the one of the standard library, so `O(n)`, where `n` is the   
    /// length of the array. (?)
    fn compute_edge_id(edge: &Vec<Node>) -> EdgeID {
        let hasher_factory = RandomState::with_seeds(SEED1, SEED2, SEED3, SEED4);
        let mut hasher = hasher_factory.build_hasher();
        edge.hash(&mut hasher);

        hasher.finish()
    }

    /// `type Node = i64`  
    ///
    /// Effectively computes the conversion of an array to an hashset.
    ///
    /// # Parameters
    /// - `array` : `&Vec<Node>` - Array to be converted.
    ///
    /// # Returns
    /// - `AHashSet<Node>` - The corresponding hashset.
    ///
    /// # Performance
    /// - `O(n)`, where `n` is the length of the array.
    fn compute_vec_to_set(array: &Vec<Node>) -> AHashSet<Node> {
        let mut res = AHashSet::new();

        for v in array.iter() {
            res.insert(*v);
        }

        res
    }
}

/*
    pub fn line_graph(&self) {}

    pub fn dual(&self) {}

    pub fn incidence_graph(&self) {}

    pub fn adjacency_list(&self) {}
*/

