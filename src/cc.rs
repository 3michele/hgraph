use ahash::AHashSet;

use super::{Hypergraph, Node};
use super::visits::_bfs;

type Component = AHashSet<Node>;

impl Hypergraph {
    /// `type Node = i64`
    /// `type Component = AHashSet<Node>`.   
    /// 
    /// Returns the connected components of the hypergraph.     
    /// 
    /// If the returned list is empty, then the hypergraph is empty, ie without nodes.
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.
    /// 
    /// # Returns 
    /// - `Result<Vec<Component>, &str>` - `Ok` containing the list of connected components (each one is a set of nodes  
    /// representing a connected subgraph of the hypergraph). Returns `Err` with a message if both `order` and `size`  
    /// are specified.
    /// 
    /// # Performance 
    /// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn ccs(&self, order: Option<usize>, size: Option<usize>) -> Result<Vec<Component>, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified.")
        } else {
            let mut visited: AHashSet<Node> = AHashSet::new();
            let mut cc = Vec::new();

            self.get_nodes().iter().for_each(|node| {
                if !visited.contains(&node) {
                    let res = _bfs(self, *node, None, None, None);
                    visited.extend(res.iter());
                    cc.push(res);
                }
            });

            Ok(cc) 
        }
    }

    /// `type Node = i64`.   
    /// `type Component = AHashSet<Node>`.   
    /// 
    /// Returns the connected component of the hypergraph containing the given node.  
    /// 
    /// If the returned set is empty, then the node is not in the hypergraph.
    /// 
    /// # Parameters 
    /// - `node` : `Node` - The node to check. 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.
    /// 
    /// # Returns 
    /// - `Result<Component, &str>` - `Ok` containing the connected component that includes the specified node (or an  
    ///  empty set if the node is not in the hypergraph). Returns `Err` with a message if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn node_cc(&self, node: Node, order: Option<usize>, size: Option<usize>) -> Result<Component, &str>{
        if order != None && size != None {
            Err("Order and size cannot be both specified.")
        } else {
            Ok(_bfs(self, node, None, order, size))
        }
    }

    /// Return the number of connected components of the hypergraph.     
    /// 
    /// If the returned number is 0, then the hypergraph is empty, ie without nodes.
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<usize, &str>` - `Ok` containing the number of connected components in the hypergraph. Returns `Err`    
    /// with a message if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn num_ccs(&self, order: Option<usize>, size: Option<usize>) -> Result<usize, &str> {
        match self.ccs(order, size) {
            Ok(val) => Ok(val.len()),
            Err(err) => Err(err)
        }
    }

    /// `type Node = i64`
    /// `type Component = AHashSet<Node>`
    /// 
    /// Return the largest connected component of the hypergraph.  
    /// 
    /// If the returned component has size 0, then the hypergraph is empty, ie without nodes.  
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<Component, &str>` - `Ok` containing the largest connected component in the hypergraph (or an empty set if   
    /// the hypergraph has no nodes). Returns `Err` with a message if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn largest_cc(&self, order: Option<usize>, size: Option<usize>) -> Result<Component, &str> {
        match self.ccs(order,size) {
            Ok(ccs) => {
                let mut res = &AHashSet::new();
                let mut res_len = 0;

                ccs.iter().for_each(|cc| {
                    if cc.len() > res_len {
                        res_len = cc.len();
                        res = cc;
                    }
                });

                Ok((*res).clone())
            }, 
            Err(err) => Err(err)
        }
    }

    /// Return the size of the largest connected component of the hypergraph.   
    /// 
    /// If the returned size is 0, then the hypergraph is empty, ie without nodes.  
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<usize, &str>` - `Ok` containing the size of the largest connected component in the hypergraph (0    
    /// if the hypergraph has no nodes). Returns `Err` with a message if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn largest_cc_size(&self, order: Option<usize>, size: Option<usize>) -> Result<usize, &str> {
        match self.largest_cc(order, size) {
            Ok(val) => {
                Ok(val.len())
            }, 
            Err(err) => Err(err)
        }
    }

    // WORKS IN O(n*m), INSTEAD OF O(n*n*m)
    /// `type Node = i64`.  
    /// 
    /// Returns the isolated nodes of the hypergraph.
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<Vec<Node>, &str>` - `Ok` containing a list of isolated nodes in the hypergraph. Returns `Err` with  
    /// a message if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively.
    pub fn isolated_nodes(&self, order: Option<usize>, size: Option<usize>) -> Result<Vec<Node>, &str> {
        if order != None && size != None {
            Err("Order and size cannot be both specified.")
        } else {
            let mut res = Vec::new();

            for node in self.incidence_list.keys() { // O(n)
                if let Ok(Some(isolated)) = self.is_isolated(*node, order, size){ //O(m)
                    if isolated {
                        res.push(*node);
                    } 
                }
            }

            Ok(res)
        }
    }

    // WORKS IN O(m), INSTEAD OF O(n*m)
    /// `type Node = i64`.  
    /// 
    /// Returns if the given node is isolated. 
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<Option<bool>, &str>` - `Ok(Some(true))` if the node is isolated, `Ok(Some(false))` if not. Returns   
    /// `Ok(None)` if the node is not found. Returns `Err` if both `order` and `size` are specified.

    /// 
    /// # Performance 
    /// - `O(m)`, where `m`is the number of hyperedges of the hypergraph. 
    pub fn is_isolated(&self, node: Node, order: Option<usize>, size: Option<usize>) -> Result<Option<bool>, &str> {
        // Both are specified
        if order != None && size != None {
            Err("Order and size cannot be both specified.")
        } else {
            match self.incidence_list.get(&node) {
                Some(edge_ids) => {
                    // None is specified 
                    if order == None && size == None {
                        for edge_id in edge_ids.iter() {
                            let hyperedge = self.edge_list.get(edge_id).unwrap(); // It will not panic
                            
                            // This could be more appropriate with hashset as edges 
                            if hyperedge.nodes.len() > 1 || (hyperedge.nodes.len() == 1 && hyperedge.nodes[0] != node) {
                                return Ok(Some(false));
                            }
                        }
                        return Ok(Some(true));
                    // Only one is specified
                    } else {
                        let filter = if let Some(val) = order {
                            val + 1
                        } else {
                            size.unwrap()
                        };

                        for edge_id in edge_ids.iter() {
                            let hyperedge = self.edge_list.get(edge_id).unwrap(); // It will not panic
                            
                            // This could be more appropriate with hashset as edges 
                            if hyperedge.nodes.len() == filter && (filter > 1 || (filter == 1 && hyperedge.nodes[0] != node)) {
                                return Ok(Some(false));
                            }
                        }
                        return Ok(Some(true));
                    }
                }, 
                _ => Ok(None)
            }
        }
    }

    // STILL O(n*n*m) IN WORST CASE, BUT IT SHOULD HALT BEFORE
    /// Returns if the given hypergraph is connected. 
    /// 
    /// # Parameters 
    /// - `order` : `Option<usize>` - The order of the hyperedges to consider. If None, all hyperedges are considered.
    /// - `size` : `Option<usize>` - The size of the hyperedges to consider. If None, all hyperedges are considered.  
    /// 
    /// # Returns 
    /// - `Result<bool, &str>` - `Ok` containing `true` if the hypergraph is connected, `false` otherwise. Returns `Err`  
    /// if both `order` and `size` are specified.
    /// 
    /// # Performance 
    /// - `O(n*n*m)`, where `n` and `m` are the number of nodes and the number of hyperedges of the hypergraph, respectively. 
    pub fn is_connected(&self, order: Option<usize>, size: Option<usize>) -> Result<bool, &str> {
        self.ccs(order, size).map_or(
            Err("Order and size cannot be both specified."),
            |components| {Ok(components.len() <= 1)}) // If the hypergraph has 0 nodes is connected by def. (?)
    }
}