use super::Node;
use std::{
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
};

/// Represents a (weighted) hyperedge in a hypergraph.  
/// 
/// A hyperedge is an edge that can link any number of nodes, as opposed to standard graph edges that only   
/// connect two nodes (see [Hypergraph](https://en.wikipedia.org/wiki/Hypergraph)).  
///
/// This struct is designed to work within a `Hypergraph` structure, where each hyperedge is uniquely   
/// identified by an `EdgeID` and associated with a concrete set of nodes.
/// 
/// # See Also
///
/// For more information on hypergraphs and how they are stored, see the documentation for `Hypergraph`.
pub struct Hyperedge {
    /// A reference-counted, mutable vector of `Node`s (node IDs) connected by this hyperedge.  
    /// This allows multiple parts of the program to share ownership of the node collection while enabling  
    /// in-place modifications when needed.
    pub nodes: Vec<Node>,

    /// Optional weight for the hyperedge.
    pub weight: f64,
}


impl Hyperedge {
    /// Create a new instance of Hyperedge.
    ///
    /// # Parameters
    /// - `nodes` : `Rc<RefCell<Vec<Node>>>` - Nodes which are incident to this hyperedge. The smart pointers are needed   
    /// to achieve multiple reference (`Rc`) and interior mutability (`RefCell`).
    /// - `weight` : `f64` - Weight of the hyperedge.
    ///
    /// # Returns  
    /// - `Self` - A new instance of `Hyperedge`.
    pub fn new(nodes: Vec<Node>, weight: f64) -> Self {
        Self { nodes, weight }
    }

    /// Change the weight of this hyperedge.
    ///
    /// # Parameters
    /// - `weight` : `f64` - The new weight.
    /// 
    /// # Returns 
    /// - `()` 
    pub fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }
}

impl Hash for Hyperedge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nodes.hash(state)
    }
}

impl PartialEq for Hyperedge {
    fn eq(&self, other: &Self) -> bool {
        self.nodes.eq(&other.nodes)
    }
}

impl Clone for Hyperedge {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(), // O(n)
            weight: self.weight,
        }
    }
}

impl Eq for Hyperedge {}

impl Display for Hyperedge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.nodes, self.weight)
    }
}

impl Debug for Hyperedge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", self.nodes, self.weight)
    }
}
