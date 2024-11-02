use std::fmt::{Debug, Display};

use super::Hypergraph;

impl Debug for Hypergraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = write!(f, "{{\n\t{:?},\n\t", self.get_nodes());

        let _ = write!(f, "[\n\t\t");

        let mut index = 0 as usize;
        for hyperedge in self.edge_list.values() {
            let _ = write!(f, "{:?}", hyperedge);
            if index + 1 < self.edge_list.values().len() {
                let _ = write!(f, ",\n\t\t");
            } else {
                let _ = write!(f, "\n\t]\n");
            }
            index += 1;
        }
        write!(f, "}}")
    }
}

impl Display for Hypergraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Hypergraph with {} nodes and {} edges",
            self.get_num_nodes(),
            self.get_num_edges()
        )
    }
}

impl Clone for Hypergraph {
    fn clone(&self) -> Self {
        Self {
            weighted: self.weighted,
            incidence_list: self.incidence_list.clone(),
            edge_list: self.edge_list.clone(),
        }
    }
}
