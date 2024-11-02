use super::Node;
use std::{
    cell::RefCell,
    fmt::{Debug, Display},
    hash::{Hash, Hasher},
    rc::Rc,
};

pub struct Hyperedge {
    pub nodes: Rc<RefCell<Vec<Node>>>,
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
    pub fn new(nodes: Rc<RefCell<Vec<Node>>>, weight: f64) -> Self {
        Self { nodes, weight }
    }

    /// Change the weight of this hyperedge.
    ///
    /// # Parameters
    /// - `weight` : `f64` - New weight of the hyperedge
    pub fn set_weight(&mut self, weight: f64) {
        self.weight = weight;
    }
}

impl Hash for Hyperedge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (*(*self.nodes).borrow()).hash(state);
    }
}

impl PartialEq for Hyperedge {
    fn eq(&self, other: &Self) -> bool {
        (&*((*self.nodes).borrow())).eq(&*(*other.nodes).borrow())
    }
}

impl Clone for Hyperedge {
    fn clone(&self) -> Self {
        Self {
            nodes: Rc::clone(&self.nodes),
            weight: self.weight,
        }
    }
}

impl Eq for Hyperedge {}

impl Display for Hyperedge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", (*self.nodes).borrow(), self.weight)
    }
}

impl Debug for Hyperedge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}, {})", (*self.nodes).borrow(), self.weight)
    }
}
